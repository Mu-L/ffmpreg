use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ffmpreg")]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
pub struct Args {
	#[arg(short, value_name = "FILE", help = "Input file")]
	pub input: String,

	#[arg(short, value_name = "FILE", help = "Output file")]
	pub output: String,
}

impl Args {
	pub fn parse() -> Self {
		<Self as clap::Parser>::parse()
	}
}
