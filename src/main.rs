extern crate colored;
extern crate regex;

use colored::Colorize;
use regex::Regex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io;

fn read(path: &str) -> Vec<String> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    data.lines().map(|l| l.to_string()).collect()
}

fn print_error(error: &str, show_verbose: bool) {
    if show_verbose {
        println!("{}", format!("? {}", error).red());
    } else {
        println!("{}", "?".red());
    }
}

fn print_line(line: &str, i: usize, n: usize, show_number: bool) {
    if show_number {
        let width = n.to_string().len();
        let number = format!("{:>width$}", i, width = width).green();
        println!("{} {}", number, line);
    } else {
        println!("{}", line);
    }
}

fn main() {
    let mut insert_mode = false;
    let mut file = None;
    let mut addr = 0;
    let mut text = Vec::new();
    let mut prompt = "";

    let mut show_debug = false;
    let mut show_verbose = false;
    let args: Vec<String> = env::args().filter(|arg| {
        if arg == "--debug" {
            show_debug = true;
        }
        if arg == "--verbose" {
            show_verbose = true;
            prompt = "> ";
        }
        !arg.starts_with("--")
    }).collect();

    if args.len() > 1 {
        file = Some(args[1].clone());
        text = read(&args[1]);
        addr = text.len();
    }

    let re = Regex::new(r"(?P<addr1>\d*)(?P<range_sep>[,;%]?)(?P<addr2>\d*)(?P<cmd>[a-z]*)(?P<cmd_sep>[ /]?)(?P<params>.*)").unwrap();

    loop {
        let mut input = String::new();
        if !insert_mode {
            print!("{}", prompt);
        }
        io::stdout().flush().ok();
        io::stdin().read_line(&mut input).expect("Unable to read stdin");

        if insert_mode {
            if input.trim() == "." {
                insert_mode = false;
            } else {
                let line = String::from(input.trim());
                text.insert(addr, line);
                addr += 1;
            }
            continue;
        }


        let caps = re.captures(input.trim()).unwrap();
        if show_debug {
            println!("# regex: {:?}", caps);
        }

        let cmd = &caps["cmd"];

        if cmd == "q" {
            break;
        }

        let cmd_sep = if &caps["cmd_sep"] == "/" { "/" } else { " " };
        let params: Vec<&str> = caps["params"].split(cmd_sep).collect();

        let mut begin = match &caps["range_sep"] {
            "," | "%" => 1,
            _         => addr
        };

        begin = match &caps["addr1"] {
            ""  => begin,
            "." => addr,
            "$" => text.len(),
            _   => caps["addr1"].parse::<usize>().unwrap()
        };

        let mut end = match &caps["range_sep"] {
            "," | "%" => text.len(),
            _         => begin
        };

        end = match &caps["addr2"] {
            ""  => end,
            "." => addr,
            "$" => text.len(),
            _   => caps["addr2"].parse::<usize>().unwrap()
        };

        addr = end;

        if begin > end || end > text.len() {
            print_error("Invalid range", show_verbose);
            continue;
        }

        let range = begin .. end + 1;
        if show_debug {
            println!("# range: [{},{}]", begin, end);
            println!("# addr: {}", addr);
            println!("# cmd: {}", cmd);
            println!("# params: {:?}", params);
        }

        match cmd {
            "" => {
                print_error("Invalid command", show_verbose);
            },
            "a" => { // Add line after addr
                insert_mode = true;
            },
            "i" => { // Insert line before addr
                insert_mode = true;
                if addr > 0 {
                    addr -= 1;
                }
            },
            "d" => {
                text.remove(addr - 1);
            },
            "o" => {
                let f = params[0];
                file = Some(String::from(f));
                text = read(params[0]);
            },
            "w" | "wq" => {
                let path = if params.len() == 1 {
                    String::from(params[0])
                } else if let Some(f) = file.clone() {
                    f
                } else {
                    print_error("No file name", show_verbose);
                    continue;
                };
                let data = text.join("\n");
                fs::write(path, data).expect("Unable to write file");
                if cmd == "wq" {
                    break;
                }
            },
            "p" |"n" | "pn" => {
                if range.start == 0 {
                    print_error("Invalid range", show_verbose);
                    continue;
                }
                let n = text.len();
                let show_number = cmd.ends_with("n");
                for i in range {
                    print_line(&text[i - 1], i, n, show_number);
                    addr = i;
                }
            },
            "g" | "gn" => {
                if range.start == 0 {
                    print_error("Invalid range", show_verbose);
                    continue;
                }
                let re = Regex::new(params[0]).unwrap();
                let n = text.len();
                let show_number = cmd.ends_with("n");
                for i in range {
                    if re.is_match(&text[i - 1]) {
                        print_line(&text[i - 1], i, n, show_number);
                        addr = i;
                    }
                }
            },
            "s" => {
                if range.start == 0 {
                    print_error("Invalid range", show_verbose);
                    continue;
                }
                let re = Regex::new(params[0]).unwrap();
                for i in range {
                    if re.is_match(&text[i - 1]) {
                        text[i - 1] = re.replace_all(&text[i - 1], params[1]).to_string();
                        addr = i;
                    }
                }
            },
            _ => {
                print_error("Invalid command", show_verbose);
            }
        }
    }
}
