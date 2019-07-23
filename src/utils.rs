use colored::Colorize;
use std::fs;

pub fn read_lines(path: &str) -> Vec<String> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    data.lines().map(|l| l.to_string()).collect()
}

pub fn print_error(error: &str, show_help: bool) {
    if show_help {
        println!("{}", format!("? {}", error).red());
    } else {
        println!("{}", "?".red());
    }
}

pub fn print_line(line: &str, i: usize, n: usize, show_number: bool) {
    if show_number {
        let width = n.to_string().len();
        let number = format!("{:>width$}", i, width = width).green();
        println!("{} {}", number, line);
    } else {
        println!("{}", line);
    }
}
