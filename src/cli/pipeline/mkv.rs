use crate::cli::pipeline::Pipeline;
use crate::container::mkv::parse_mkv;
use crate::io;

pub fn run(pipe: Pipeline) -> io::Result<()> {
	let input_data = std::fs::read(&pipe.input).map_err(|e| {
		io::Error::with_message(io::ErrorKind::Other, format!("failed to read input: {}", e))
	})?;

	let mkv = match parse_mkv(&input_data) {
		Ok(mkv) => mkv.1,
		Err(e) => {
			return Err(io::Error::with_message(
				io::ErrorKind::Other,
				format!("failed to parse mkv: {}", e),
			));
		}
	};

	println!("File: {}", pipe.input);
	mkv.tracks.print_summary();

	Ok(())
}
