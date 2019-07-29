mod addresses;
mod commands;
mod editor;
mod utils;

use crate::editor::*;
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
        let res = ed.exec(&format!("e {}", args[1]));

        if let Err(e) = res {
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
                match ed.exec(input.trim_end()) {
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
