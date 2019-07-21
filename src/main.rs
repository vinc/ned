extern crate colored;
extern crate regex;
extern crate rustyline;

use colored::Colorize;
use regex::Regex;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::env;
use std::fs;

fn read_lines(path: &str) -> Vec<String> {
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
    let home = std::env::var("HOME").unwrap();
    let history = format!("{}/.ned_history", home);

    let mut insert_mode = false;
    let mut lines = Vec::new();
    let mut addr = 0;
    let mut file = None;
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
        let f = args[1].clone();
        lines = read_lines(&f);
        addr = lines.len();
        file = Some(String::from(f));
    }

    let re = Regex::new(r"(?P<addr1>\d*)(?P<range_sep>[,;%]?)(?P<addr2>\d*)(?P<cmd>[a-z]*)(?P<cmd_sep>[ /]?)(?P<params>.*)").unwrap();

    let mut rl = Editor::<()>::new();
    rl.load_history(&history).ok();

    loop {
        let readline = rl.readline(if insert_mode { "" } else { prompt });
        match readline {
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            },
            Ok(input) => {
                rl.add_history_entry(input.as_str());

                if insert_mode {
                    if input == "." {
                        insert_mode = false;
                    } else {
                        lines.insert(addr, input);
                        addr += 1;
                    }
                    continue;
                }
                let caps = re.captures(&input).unwrap();
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
                    "$" => lines.len(),
                    _   => caps["addr1"].parse::<usize>().unwrap()
                };

                let mut end = match &caps["range_sep"] {
                    "," | "%" => lines.len(),
                    _         => begin
                };

                end = match &caps["addr2"] {
                    ""  => end,
                    "." => addr,
                    "$" => lines.len(),
                    _   => caps["addr2"].parse::<usize>().unwrap()
                };

                addr = end;

                if begin > end || end > lines.len() {
                    print_error("Invalid range", show_verbose);
                    continue;
                }

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
                    "c" => { // Change lines
                        let range = begin - 1 .. end;
                        lines.drain(range);
                        addr = begin - 1;
                        insert_mode = true;
                    },
                    "d" => { // Delete lines
                        let range = begin - 1 .. end;
                        lines.drain(range);
                    },
                    "e" => { // Open file
                        let f = params[0];
                        lines = read_lines(&f);
                        addr = lines.len();
                        file = Some(String::from(f));
                    },
                    "w" | "wq" => { // Write to file (and quit)
                        if params[0] != "" {
                            file = Some(String::from(params[0]));
                        }
                        if let Some(f) = file.clone() {
                            let data = lines.join("\n");
                            fs::write(f, data).expect("Unable to write file");
                            if cmd == "wq" {
                                break;
                            }
                        } else {
                            print_error("No file name", show_verbose);
                            continue;
                        }
                    },
                    "p" |"n" | "pn" => { // Print file (with numbered lines)
                        if begin == 0 {
                            print_error("Invalid range", show_verbose);
                            continue;
                        }
                        let range = begin .. end + 1;
                        let n = lines.len();
                        let show_number = cmd.ends_with("n");
                        for i in range {
                            print_line(&lines[i - 1], i, n, show_number);
                            addr = i;
                        }
                    },
                    "g" => { // Global command
                        if begin == 0 {
                            print_error("Invalid range", show_verbose);
                            continue;
                        }
                        let re = Regex::new(params[0]).unwrap();
                        let cmd_list = if params.len() == 2 { params[1] } else { "p" };
                        let show_number = cmd_list.ends_with("n");
                        let mut i = begin;
                        let mut n = end;
                        while i < n {
                            if re.is_match(&lines[i - 1]) {
                                match cmd_list {
                                    "p" | "pn" => {
                                        print_line(&lines[i - 1], i, lines.len(), show_number);
                                    },
                                    "d" => {
                                        lines.remove(i - 1);
                                        n -= 1;
                                    },
                                    _ => {}
                                }
                                addr = i;
                            }
                            i += 1;
                        }
                    },
                    "s" => { // Substitute command
                        if begin == 0 {
                            print_error("Invalid range", show_verbose);
                            continue;
                        }
                        let range = begin .. end + 1;
                        let re = Regex::new(params[0]).unwrap();
                        for i in range {
                            if re.is_match(&lines[i - 1]) {
                                lines[i - 1] = re.replace_all(&lines[i - 1], params[1]).to_string();
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
    }
    rl.save_history(&history).unwrap();
}
