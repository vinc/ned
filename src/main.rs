mod editor;
mod addresses;
mod commands;
mod utils;

use crate::editor::*;
use crate::addresses::*;
use crate::commands::*;
use crate::utils::*;

use rustyline::error::ReadlineError;
use std::env;

static PROMPT: &str = "> ";

fn main() {
    let home = std::env::var("HOME").unwrap();
    let history = format!("{}/.ned_history", home);

    let mut ed = Editor::new();
    let mut prompt = PROMPT;
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
        let cl = CommandLine {
            addr_1: None,
            addr_2: None,
            cmd: "e".to_string(),
            flag: false,
            params: vec![filename]
        };
        if let Err(e) = ed.edit_command(cl) {
            print_error(e, ed.show_help);
        }
    }

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

                let mut cl = ed.parse_command_line(input);

                if !ed.is_range_ok(cl.clone()) {
                    print_error(Error::InvalidAddress, ed.show_help);
                    continue;
                }
                if cl.addr_1.is_none() {
                    cl.addr_1 = Some(ed.addr);
                }
                if cl.addr_2.is_none() {
                    cl.addr_2 = cl.addr_1;
                }

                if ed.show_debug {
                    println!("# range: [{:?},{:?}]", cl.addr_1, cl.addr_2);
                    println!("# addr: {}", ed.addr);
                    println!("# cmd: {}", cl.cmd);
                    println!("# params: {:?}", cl.params);
                }

                let res = match cl.cmd.as_str() {
                    "a" => ed.append_command(cl), // insert [a]fter
                    "b" => ed.insert_command(cl), // insert [b]efore
                    "i" => ed.insert_command(cl), // [i]nsert before
                    "c" => ed.change_command(cl), // [d] + [i]
                    "d" => ed.delete_command(cl),
                    "e" => ed.edit_command(cl),
                    "f" => ed.filename_command(cl),
                    "w" => ed.write_command(cl),
                    "p" => ed.print_command(cl),
                    "n" => ed.number_command(cl),
                    "g" => ed.global_command(cl),
                    "s" => ed.substitute_command(cl),
                    "q" => ed.quit_command(cl),
                    "x" => ed.write_and_quit_command(cl), // [w] + [q]
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
