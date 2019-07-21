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
    let mut dirty = false;

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

    let re = Regex::new(concat!(
        "(?P<first_addr>[0-9]*)",
        "(?P<range_sep>[,;%]?)",
        "(?P<last_addr>[0-9]*)",
        "(?P<cmd>[a-zA-Z]*)",
        "(?P<cmd_sep>[ /]?)",
        "(?P<params>.*)"
    )).unwrap();

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
                let input = input.trim();

                if insert_mode {
                    if input == "." {
                        insert_mode = false;
                    } else {
                        lines.insert(addr, input.to_string());
                        addr += 1;
                        dirty = true;
                    }
                    continue;
                } else {
                    rl.add_history_entry(input);
                }

                let caps = re.captures(input).unwrap();
                if show_debug {
                    println!("# regex: {:?}", caps);
                }

                let cmd = &caps["cmd"];

                let cmd_sep = if &caps["cmd_sep"] == "/" { "/" } else { " " };
                let params: Vec<&str> = caps["params"].split(cmd_sep).collect();

                let mut first_addr = match &caps["range_sep"] {
                    "," | "%" => 1,
                    _         => addr
                };

                first_addr = match &caps["first_addr"] {
                    ""  => first_addr,
                    "." => addr,
                    "$" => lines.len(),
                    _   => caps["first_addr"].parse::<usize>().unwrap()
                };

                let mut last_addr = match &caps["range_sep"] {
                    "," | "%" => lines.len(),
                    _         => first_addr
                };

                last_addr = match &caps["last_addr"] {
                    ""  => last_addr,
                    "." => addr,
                    "$" => lines.len(),
                    _   => caps["last_addr"].parse::<usize>().unwrap()
                };

                addr = last_addr;

                if first_addr > last_addr || last_addr > lines.len() {
                    if first_addr != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                        print_error("Invalid range", show_verbose);
                        continue;
                    }
                }

                if show_debug {
                    println!("# range: [{},{}]", first_addr, last_addr);
                    println!("# addr: {}", addr);
                    println!("# cmd: {}", cmd);
                    println!("# params: {:?}", params);
                }

                match cmd {
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
                        let range = first_addr - 1 .. last_addr;
                        lines.drain(range);
                        addr = first_addr - 1;
                        dirty = true;
                        insert_mode = true;
                    },
                    "d" => { // Delete lines
                        let range = first_addr - 1 .. last_addr;
                        lines.drain(range);
                        dirty = true;
                        addr = first_addr - 1;
                    },
                    "e" => { // Open file
                        let f = params[0];
                        lines = read_lines(&f);
                        addr = lines.len();
                        file = Some(String::from(f));
                        dirty = false;
                    },
                    "w" | "wq" => { // Write to file (and quit)
                        if params[0] != "" {
                            file = Some(String::from(params[0]));
                        }
                        if let Some(f) = file.clone() {
                            let data = lines.join("\n");
                            fs::write(f, data).expect("Unable to write file");
                            dirty = false;
                        } else {
                            print_error("No file name", show_verbose);
                            continue;
                        }
                    },
                    "p" |"n" | "pn" => { // Print file (with numbered lines)
                        let range = first_addr .. last_addr + 1;
                        let n = lines.len();
                        let show_number = cmd.ends_with("n");
                        for i in range {
                            print_line(&lines[i - 1], i, n, show_number);
                            addr = i;
                        }
                    },
                    "g" => { // Global command
                        let re = Regex::new(params[0]).unwrap();
                        let cmd_list = if params.len() == 2 { params[1] } else { "p" };
                        let show_number = cmd_list.ends_with("n");
                        let mut i = first_addr;
                        let mut n = last_addr;
                        while i <= n {
                            if re.is_match(&lines[i - 1]) {
                                match cmd_list {
                                    "p" | "pn" => {
                                        print_line(&lines[i - 1], i, lines.len(), show_number);
                                    },
                                    "d" => {
                                        lines.remove(i - 1);
                                        i -= 1;
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
                        let range = first_addr .. last_addr + 1;
                        let re = Regex::new(params[0]).unwrap();
                        for i in range {
                            if re.is_match(&lines[i - 1]) {
                                lines[i - 1] = re.replace_all(&lines[i - 1], params[1]).to_string();
                                addr = i;
                            }
                        }
                    },
                    "q" => {
                        if dirty {
                            print_error("No write since last change", show_verbose);
                            continue;
                        }
                    },
                    "Q" => {
                        // Nothing to do here
                    },
                    _ => {
                        print_error("Invalid command", show_verbose);
                    }
                }

                if cmd.to_lowercase().ends_with("q") {
                    break;
                }
            }
        }
    }
    rl.save_history(&history).unwrap();
}
