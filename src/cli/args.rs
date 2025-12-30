use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ffmpreg")]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
pub struct Cli {
	#[arg(short, long)]
	pub input: String,

	#[arg(short, long)]
	pub output: String,

	#[arg(long, num_args = 1..)]
	pub audio: Vec<String>,

	#[arg(long, num_args = 1..)]
	pub video: Vec<String>,

	#[arg(long, num_args = 1..)]
	pub subtitle: Vec<String>,

	#[arg(long, num_args = 1..)]
	pub apply: Vec<String>,
}
