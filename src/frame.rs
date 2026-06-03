use core::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

use bytes::{Buf, Bytes};

type CursorType = &mut Cursor<&[u8]>;

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl Frame {
    pub(crate) fn array() -> Self {
        Self::Array(Vec::new())
    }

    pub(crate) fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Self::Array(vec) => {
                vec.push(Frame::Bulk(bytes));
            }
            _ => panic!("push_bulk called on non-array frame"),
        }
    }

    pub(crate) fn push_int(&mut self, value: u64) {
        match self {
            Self::Array(vec) => {
                vec.push(Frame::Integer(value));
            }
            _ => panic!("push_int called on non-array frame"),
        }
    }

    pub fn check(src: CursorType) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b':' => {
                let _ = get_decimal(src)?;
                Ok(())
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err("protocol error: expected -1".into());
                    }
                    Ok(())
                } else {
                    let len: usize = get_decimal(src)?.try_into()?;
                    skip(src, len + 2)
                }
            }
            b'*' => {
                let len = get_decimal(src)?;
                for _ in 0..len {
                    Self::check(src)?;
                }
                Ok(())
            }
            actual => Err(format!("protocol error: unexpected first byte: {}", actual).into()),
        }
    }

    pub fn parse(src: CursorType) -> Result<Self, Error> {
        match get_u8(src)? {
            b'+' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Self::Simple(string))
            }
            b'-' => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;
                Ok(Self::Error(string))
            }
            b':' => {
                let len = get_decimal(src)?;
                Ok(Self::Integer(len))
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;

                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }

                    Ok(Frame::Null)
                } else {
                    let len = get_decimal(src)?.try_into()?;
                    let n = len + 2;

                    if src.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                    skip(src, n)?;

                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);
                for _ in 0..len {
                    out.push(Self::parse(src)?);
                }
                Ok(Self::Array(out))
            }
            _ => unimplemented!(),
        }
    }

    pub(crate) fn to_error(&self) -> crate::Error {
        format!("unexpected frame: {self}").into()
    }
}

fn peek_u8(src: CursorType) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.chunk()[0])
}

fn get_u8(src: CursorType) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

fn skip(src: CursorType, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }
    src.advance(n);
    Ok(())
}

fn get_decimal(src: CursorType) -> Result<u64, Error> {
    use atoi::atoi;
    let line = get_line(src)?;
    atoi::<u64>(line).ok_or_else(|| "protocol error: invalid decimal".into())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;
    let buf = src.get_ref();

    if let Some(i) = (start..end).find(|&i| buf[i] == b'\r' && buf[i + 1] == b'\n') {
        src.set_position((i + 2) as u64);
        return Ok(&buf[start..i]);
    }

    Err(Error::Incomplete)
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Simple(response) => response.fmt(fmt),
            Frame::Error(msg) => write!(fmt, "error: {msg}"),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{msg:?}"),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, " ")?;
                    }

                    part.fmt(fmt)?;
                }

                Ok(())
            }
        }
    }
}

impl From<String> for Error {
    fn from(src: String) -> Self {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Self {
        Error::Other(src.into())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Self {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Self {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(f),
            Error::Other(err) => err.fmt(f),
        }
    }
}
