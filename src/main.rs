use ffmpreg::cli::{Args, Pipeline};

fn main() {
	let args = Args::parse();
	let pipeline = Pipeline::new(args.input.clone(), args.output.clone());
	if let Err(e) = pipeline.run() {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
	println!("Success: {} -> {}", args.input, args.output);
}
