use clap::Parser;
use ffmpreg::cli::color;
use ffmpreg::cli::{Cli, executor};
use ffmpreg::{EXIT_FAILURE, EXIT_SUCCESS};

fn main() {
	let cli = Cli::parse();
	if let Err(message) = executor::execute(cli) {
		color::print_error(&message);
		std::process::exit(EXIT_FAILURE);
	}
	color::print_success(None);
	std::process::exit(EXIT_SUCCESS);
}
