pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_RED: &str = "\x1b[31m";
pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_BLUE: &str = "\x1b[34m";
pub const COLOR_MAGENTA: &str = "\x1b[35m";
pub const COLOR_CYAN: &str = "\x1b[36m";
pub const COLOR_WHITE: &str = "\x1b[37m";

pub fn print_error(message: impl std::fmt::Display) {
	let message = format!("{}{}{}", COLOR_WHITE, message, COLOR_RESET);
	let tag = format!("{}error: {}", COLOR_RED, COLOR_RESET);
	println!("{}{}", tag, message);
}

pub fn print_warning(message: impl std::fmt::Display) {
	let message = format!("{}{}{}", COLOR_WHITE, message, COLOR_RESET);
	let tag = format!("{}warning: {}", COLOR_YELLOW, COLOR_RESET);
	println!("{}{}", tag, message);
}

pub fn print_success(message: Option<String>) {
	if let Some(message) = message {
		let message = format!("{}{}{}", COLOR_WHITE, message, COLOR_RESET);
		let tag = format!("{}success: {}", COLOR_GREEN, COLOR_RESET);
		println!("{}{}", tag, message);
	}
	let tag = format!("{}done.{}", COLOR_GREEN, COLOR_RESET);
	println!("{}{}", tag, COLOR_RESET);
}
