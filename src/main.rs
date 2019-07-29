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
    let mut undo = vec![];
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
        let cl = CommandLine {
            addr_1: None,
            addr_2: None,
            cmd: "e".to_string(),
            flag: false,
            params: vec![args[1].clone()]
        };
        if let Err(e) = ed.edit_command(cl) {
            print_error(e, ed.show_help);
        }
        ed.log(&format!("e {}", args[1].clone()));
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
                ed.log(input);

                if ed.insert_mode {
                    if input == "." {
                        ed.insert_mode = false;
                    } else {
                        ed.lines.insert(ed.addr, input.to_string());
                        ed.addr += 1;
                        ed.dirty = true;
                    }
                    continue;
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

                if cl.is_undoable() {
                    undo.push(ed.clone());
                } else if cl.is_edit() {
                    undo.clear();
                }

                let res = if cl.is_undo() {
                    match undo.pop() {
                        Some(previous_state) => {
                            ed = previous_state;
                            Ok(State::Running)
                        },
                        None => {
                            Err(Error::NoUndo)
                        }
                    }
                } else {
                    ed.exec(cl)
                };

                match res {
                    Err(error) => {
                        print_error(error, ed.show_help)
                    },
                    Ok(State::Stopped) => {
                        rl.add_history_entry(input);
                        break
                    }
                    Ok(State::Running) => {
                        rl.add_history_entry(input);
                    },
                }
            }
        }
    }
    rl.save_history(&history).unwrap();
}
