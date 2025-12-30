pub mod bit;
pub mod decoder;
pub mod encoder;
pub mod parser;
pub mod utils;

pub use decoder::AACDecoder;
pub use encoder::AACEncoder;
pub use parser::{ADTSHeader, ADTSParser};
