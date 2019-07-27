use crate::utils::*;
use crate::editor::*;
use std::fs;
use regex::Regex;

pub trait Commands {
    fn append_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn insert_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn change_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn delete_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn edit_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn filename_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn write_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn print_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn number_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn global_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn substitute_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn quit_command(&self, cl: CommandLine) -> Result<State, Error>;
    fn write_and_quit_command(&mut self, cl: CommandLine) -> Result<State, Error>;
    fn invalid_command(&self) -> Result<State, Error>;
}

impl Commands for Editor {
    fn append_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        self.addr = cl.addr_1.unwrap();
        self.insert_mode = true;
        Ok(State::Running)
    }

    fn insert_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        self.addr = cl.addr_1.unwrap();
        self.insert_mode = true;
        if self.addr > 0 {
            self.addr -= 1;
        }
        Ok(State::Running)
    }

    fn change_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        self.delete_command(cl.clone()).ok();
        self.insert_command(cl.clone())
    }

    fn delete_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        self.lines.drain(cl.addr_1.unwrap() - 1 .. cl.addr_2.unwrap());
        self.addr = cl.addr_1.unwrap();
        self.dirty = true;
        Ok(State::Running)
    }

    fn edit_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        if cl.params.len() == 0 {
            return Err(Error::NoFilename);
        }
        let filename = cl.params[0].clone();
        self.filename = Some(filename.clone());

        match read_lines(&filename) {
            Err(error) => {
                return Err(error);
            },
            Ok(lines) => {
                self.lines = lines;
                self.addr = self.lines.len();
                self.dirty = false;
            }
        }
        Ok(State::Running)
    }

    fn filename_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        if cl.params.len() == 1 {
            self.filename = Some(cl.params[0].clone());
        } else if let Some(f) = self.filename.clone() {
            println!("{}", f);
        } else {
            return Err(Error::NoFilename);
        }
        Ok(State::Running)
    }

    fn write_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        if cl.params.len() == 1 {
            self.filename = Some(cl.params[0].clone());
        }

        if let Some(f) = self.filename.clone() {
            let data = self.lines.join("\n") + "\n";
            fs::write(f, data).expect("Unable to write file");
            self.dirty = false;
            Ok(State::Running)
        } else {
            return Err(Error::NoFilename);
        }
    }

    fn print_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        let n = self.lines.len();
        for i in cl.addr_1.unwrap() .. cl.addr_2.unwrap() + 1 {
            print_line(&self.lines[i - 1], i, n, false);
            self.addr = i;
        }
        Ok(State::Running)
    }

    fn number_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        let n = self.lines.len();
        for i in cl.addr_1.unwrap() .. cl.addr_2.unwrap() + 1 {
            print_line(&self.lines[i - 1], i, n, true);
            self.addr = i;
        }
        Ok(State::Running)
    }

    fn global_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        let re = Regex::new(&cl.params[0]).unwrap();
        let cmd_list = if cl.params.len() == 2 { &cl.params[1] } else { "p" };
        let show_number = cmd_list.ends_with("n");
        let mut i = cl.addr_1.unwrap();
        let mut n = cl.addr_2.unwrap();
        while i <= n {
            if re.is_match(&self.lines[i - 1]) {
                match cmd_list {
                    "p" | "n" | "pn" => {
                        print_line(&self.lines[i - 1], i, self.lines.len(), show_number);
                    },
                    "d" => {
                        self.lines.remove(i - 1);
                        i -= 1;
                        n -= 1;
                    },
                    _ => {}
                }
                self.addr = i;
            }
            i += 1;
        }
        Ok(State::Running)
    }

    fn substitute_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        let re = Regex::new(&cl.params[0]).unwrap();
        let limit = if cl.params.len() == 3 {
            if &cl.params[2] == "g" {
                0
            } else {
                cl.params[2].parse::<usize>().unwrap()
            }
        } else {
            1
        };
        for i in cl.addr_1.unwrap() .. cl.addr_2.unwrap() + 1 {
            if re.is_match(&self.lines[i - 1]) {
                // NOTE: This will replace at most <limit> matches, whereas ed
                // would replace the <limit> nth match.
                self.lines[i - 1] = re.replacen(&self.lines[i - 1], limit, cl.params[1].as_str()).to_string();
                self.addr = i;
                self.dirty = true;
            }
        }
        Ok(State::Running)
    }

    fn quit_command(&self, cl: CommandLine) -> Result<State, Error> {
        if self.dirty && !cl.flag{
            Err(Error::Dirty)
        } else {
            Ok(State::Stopped)
        }
    }

    fn write_and_quit_command(&mut self, cl: CommandLine) -> Result<State, Error> {
        match self.write_command(cl) {
            Ok(_) => Ok(State::Stopped),
            Err(error) => Err(error)
        }
    }

    fn invalid_command(&self) -> Result<State, Error> {
        Err(Error::InvalidCommand)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::addresses::*;

    static TEST_FILE: &str = "LICENSE";
    static TEST_FILE_LENGTH: usize = 21;

    // The a command shall read the given text and append it after the addressed line; the current
    // line number shall become the address of the last inserted line or, if there were none, the
    // addressed line. Address 0 shall be valid for this command; it shall cause the appended text
    // to be placed at the beginning of the buffer.
    #[test]
    fn test_append_command() {
        let mut ed = Editor::new();
        let mut cl = CommandLine {
            addr_1: None,
            addr_2: None,
            cmd: "e".to_string(),
            flag: false,
            params: vec![TEST_FILE.to_string()]
        };
        ed.edit_command(cl.clone()).ok();

        /*
        assert_eq!(ed.is_range_ok(0, 0, "a"), true);
        assert_eq!(ed.is_range_ok(1, 1, "a"), true);
        assert_eq!(ed.is_range_ok(TEST_FILE_LENGTH, TEST_FILE_LENGTH, "a"), true);
        assert_eq!(ed.is_range_ok(TEST_FILE_LENGTH + 1, TEST_FILE_LENGTH + 1,"a"), false);
        */

        cl.cmd = "a".to_string();
        cl.addr_1 = Some(0);
        assert_eq!(ed.append_command(cl.clone()), Ok(State::Running));
        assert_eq!(ed.addr, 0);
        assert_eq!(ed.insert_mode, true);

        cl.addr_1 = Some(1);
        assert_eq!(ed.append_command(cl.clone()), Ok(State::Running));
        assert_eq!(ed.addr, 1);
        assert_eq!(ed.insert_mode, true);
    }
}
