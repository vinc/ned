mod editor;
mod addresses;
mod commands;
mod utils;

use crate::editor::*;
use crate::addresses::*;
use crate::commands::*;
use crate::utils::*;

use regex::Regex;
use rustyline::error::ReadlineError;
use std::env;

fn main() {
    let home = std::env::var("HOME").unwrap();
    let history = format!("{}/.ned_history", home);

    let mut ed = Editor::new();
    let mut prompt = "> ";
    let args: Vec<String> = env::args().filter(|arg| {
        if arg == "--debug" {
            ed.show_debug = true;
        }
        if arg == "--silent" {
            ed.show_help = false;
            prompt = "";
        }
        !arg.starts_with("--")
    }).collect();

    if args.len() == 2 {
        let filename = args[1].clone();
        let params: Vec<&str> = vec![&filename];
        if let Err(e) = ed.edit_command(params) {
            print_error(e, ed.show_help);
        }
    }

    let re = Regex::new(concat!(
        "(?P<addr_1>[0-9]*)",
        "(?P<addr_sep>[,;%]?)",
        "(?P<addr_2>[0-9]*)",
        "(?P<cmd>[a-zA-Z]*)",
        "(?P<cmd_sep>[ /]?)",
        "(?P<params>.*)"
    )).unwrap();

    let mut rl = rustyline::Editor::<()>::new();
    rl.load_history(&history).ok();

    loop {
        let readline = rl.readline(if ed.insert_mode { "" } else { prompt });
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
                let input = input.trim_end();

                if ed.insert_mode {
                    if input == "." {
                        ed.insert_mode = false;
                    } else {
                        ed.lines.insert(ed.addr, input.to_string());
                        ed.addr += 1;
                        ed.dirty = true;
                    }
                    continue;
                } else {
                    rl.add_history_entry(input);
                }

                let caps = re.captures(input).unwrap();

                let cmd = &caps["cmd"];
                let cmd_sep = if &caps["cmd_sep"] == "/" { "/" } else { " " };
                let params: Vec<&str> = caps["params"].split(cmd_sep).collect();

                let (addr_1, addr_2) = ed.parse_addresses(&caps["addr_1"], &caps["addr_sep"], &caps["addr_2"]);

                if !ed.is_range_ok(addr_1, addr_2, cmd) {
                    print_error(Error::InvalidAddress, ed.show_help);
                    continue;
                }

                if ed.show_debug {
                    println!("# range: [{},{}]", addr_1, addr_2);
                    println!("# addr: {}", ed.addr);
                    println!("# cmd: {}", cmd);
                    println!("# params: {:?}", params);
                }

                let res = match cmd {
                    "a" => ed.append_command(addr_1),
                    "i" => ed.insert_command(addr_1),
                    "c" => ed.change_command(addr_1, addr_2),
                    "d" => ed.delete_command(addr_1, addr_2),
                    "e" => ed.edit_command(params),
                    "f" => ed.filename_command(params),
                    "w" => ed.write_command(params),
                    "p" => ed.print_command(addr_1, addr_2),
                    "n" => ed.number_command(addr_1, addr_2),
                    "g" => ed.global_command(addr_1, addr_2, params),
                    "s" => ed.substitute_command(addr_1, addr_2, params),
                    "q" => ed.quit_command(),
                    "Q" => ed.quit_without_checking_command(),
                    _   => ed.invalid_command()
                };

                match res {
                    Ok(State::Running) => { continue },
                    Ok(State::Stopped) => { break },
                    Err(error) => { print_error(error, ed.show_help) }
                }
            }
        }
    }
    rl.save_history(&history).unwrap();
}
