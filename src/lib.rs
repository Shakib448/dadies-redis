mod frame;
mod parser;

pub use frame::Frame;
pub use parser::Parse;

pub const DEFAULT_PORT: u16 = 5000;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
