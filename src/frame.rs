use std::io::Cursor;

use bytes::{Buf, Bytes};

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

    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => Ok(()),
            b'-' => Ok(()),
            b':' => Ok(()),
            b'$' => Ok(()),
            b'*' => Ok(()),
            actual => Err(format!("protocol error: unexpected first byte: {}", actual).into()),
        }
    }
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }
    src.advance(n);
    Ok(())
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
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
