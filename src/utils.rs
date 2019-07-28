use crate::editor::Error;
use colored::Colorize;
use std::fs;

static COLOR_ERRORS: &str = "red";
static COLOR_NUMBERS: &str = "yellow";

pub fn print_error(error: Error, show_help: bool) {
    let msg = match error {
        Error::InvalidCommand => "Invalid command",
        Error::InvalidAddress => "Invalid address",
        Error::NoFilename => "No file name",
        Error::NoUndo => "Nothing to undo",
        Error::CannotOpen => "Cannot open input file",
        Error::Dirty => "No write since last change"
    };

    if show_help {
        println!("{}", format!("? {}", msg).bold().color(COLOR_ERRORS));
    } else {
        println!("{}", "?".bold().color(COLOR_ERRORS));
    }
}

pub fn print_line(line: &str, i: usize, n: usize, show_number: bool) {
    if show_number {
        let width = n.to_string().len();
        let number = format!("{:>width$}", i, width = width).color(COLOR_NUMBERS);
        println!("{} {}", number, line);
    } else {
        println!("{}", line);
    }
}

pub fn read_lines(path: &str) -> Result<Vec<String>, Error> {
    match fs::read_to_string(path) {
        Err(e) => {
            println!("{}", e);
            Err(Error::CannotOpen)
        },
        Ok(data) => {
            Ok(data.lines().map(|l| l.to_string()).collect())
        }
    }
}
