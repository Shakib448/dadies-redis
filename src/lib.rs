mod connection;
mod frame;
mod parser;
mod shutdown;

pub use frame::Frame;
use parser::{Parse, ParseError};

pub const DEFAULT_PORT: u16 = 5000;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
