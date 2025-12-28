mod args;
pub mod executor;
// pub mod pipeline;
pub mod color;
pub mod sink;
pub mod source;
pub mod track;
pub mod transcoder;
mod utils;

pub use args::Cli;
pub use sink::Sink;

// pub fn parse_intent(cli: Cli) -> Result<intent::Intent> {}
