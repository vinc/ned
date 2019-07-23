extern crate colored;
extern crate regex;
extern crate rustyline;

use colored::Colorize;
use regex::Regex;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::env;
use std::fs;

static INVALID_COMMAND_ERROR: &str = "Invalid command";
static INVALID_RANGE_ERROR: &str = "Invalid range";
static NO_FILENAME_ERROR: &str = "No file name";
static DIRTY_ERROR: &str = "No write since last change";

fn read_lines(path: &str) -> Vec<String> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    data.lines().map(|l| l.to_string()).collect()
}

fn print_error(error: &str, show_help: bool) {
    if show_help {
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

    // State
    let mut dirty = false;
    let mut insert_mode = false;
    let mut filename = None;
    let mut lines = Vec::new();
    let mut addr = 0;

    let mut prompt = "> ";
    let mut show_help = true;
    let mut show_debug = false;
    let args: Vec<String> = env::args().filter(|arg| {
        if arg == "--debug" {
            show_debug = true;
        }
        if arg == "--silent" {
            show_help = false;
            prompt = "";
        }
        !arg.starts_with("--")
    }).collect();

    if args.len() > 1 {
        let f = args[1].clone();
        lines = read_lines(&f);
        addr = lines.len();
        filename = Some(String::from(f));
    }

    let re = Regex::new(concat!(
        "(?P<addr_1>[0-9]*)",
        "(?P<addr_sep>[,;%]?)",
        "(?P<addr_2>[0-9]*)",
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

                let cmd = &caps["cmd"];
                let cmd_sep = if &caps["cmd_sep"] == "/" { "/" } else { " " };
                let params: Vec<&str> = caps["params"].split(cmd_sep).collect();

                let mut addr_1 = match &caps["addr_sep"] {
                    "," | "%" => 1,
                    _         => addr
                };

                addr_1 = match &caps["addr_1"] {
                    ""  => addr_1,
                    "." => addr,
                    "$" => lines.len(),
                    _   => caps["addr_1"].parse::<usize>().unwrap()
                };

                let mut addr_2 = match &caps["addr_sep"] {
                    "," | "%" => lines.len(),
                    _         => addr_1
                };

                addr_2 = match &caps["addr_2"] {
                    ""  => addr_2,
                    "." => addr,
                    "$" => lines.len(),
                    _   => caps["addr_2"].parse::<usize>().unwrap()
                };

                addr = addr_2;

                if addr_1 > addr_2 || addr_2 > lines.len() {
                    if addr_1 != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                        print_error(INVALID_RANGE_ERROR, show_help);
                        continue;
                    }
                }

                if show_debug {
                    println!("# range: [{},{}]", addr_1, addr_2);
                    println!("# addr: {}", addr);
                    println!("# cmd: {}", cmd);
                    println!("# params: {:?}", params);
                }

                match cmd {
                    "a" => { // Append command
                        insert_mode = true;
                    },
                    "i" => { // Insert command
                        insert_mode = true;
                        if addr > 0 {
                            addr -= 1;
                        }
                    },
                    "c" => { // Change command
                        let range = addr_1 - 1 .. addr_2;
                        lines.drain(range);
                        addr = addr_1 - 1;
                        dirty = true;
                        insert_mode = true;
                    },
                    "d" => { // Delete command
                        let range = addr_1 - 1 .. addr_2;
                        lines.drain(range);
                        dirty = true;
                        addr = addr_1 - 1;
                    },
                    "e" => { // Edit command
                        let f = params[0];
                        lines = read_lines(&f);
                        addr = lines.len();
                        filename = Some(String::from(f));
                        dirty = false;
                    },
                    "f" => { // Filename command
                        if params[0] != "" {
                            filename = Some(String::from(params[0]));
                        } else if let Some(f) = filename.clone() {
                            println!("{}", f);
                        } else {
                            print_error(NO_FILENAME_ERROR, show_help);
                        }
                    },
                    "w" | "wq" => { // Write command
                        if params[0] != "" {
                            filename = Some(String::from(params[0]));
                        }
                        if let Some(f) = filename.clone() {
                            let data = lines.join("\n");
                            fs::write(f, data).expect("Unable to write file");
                            dirty = false;
                        } else {
                            print_error(NO_FILENAME_ERROR, show_help);
                            continue;
                        }
                    },
                    "p" |"n" | "pn" => { // Print and number commands
                        let range = addr_1 .. addr_2 + 1;
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
                        let mut i = addr_1;
                        let mut n = addr_2;
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
                        let range = addr_1 .. addr_2 + 1;
                        let re = Regex::new(params[0]).unwrap();
                        for i in range {
                            if re.is_match(&lines[i - 1]) {
                                lines[i - 1] = re.replace_all(&lines[i - 1], params[1]).to_string();
                                addr = i;
                                dirty = true;
                            }
                        }
                    },
                    "q" => { // Quit command
                        if dirty {
                            print_error(DIRTY_ERROR, show_help);
                            continue;
                        }
                    },
                    "Q" => { // Quit without checking command
                        // Nothing to do here
                    },
                    _ => {
                        print_error(INVALID_COMMAND_ERROR, show_help);
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
