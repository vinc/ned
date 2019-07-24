use crate::utils::*;
use crate::editor::*;
use std::fs;
use regex::Regex;

pub trait Commands {
    fn append_command(&mut self, addr_1: usize) -> Result<State, Error>;
    fn insert_command(&mut self, addr_1: usize) -> Result<State, Error>;
    fn change_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error>;
    fn delete_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error>;
    fn edit_command(&mut self, params: Vec<&str>) -> Result<State, Error>;
    fn filename_command(&mut self, params: Vec<&str>) -> Result<State, Error>;
    fn write_command(&mut self, params: Vec<&str>) -> Result<State, Error>;
    fn print_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error>;
    fn number_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error>;
    fn global_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> Result<State, Error>;
    fn substitute_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> Result<State, Error>;
    fn quit_command(&self) -> Result<State, Error>;
    fn quit_without_checking_command(&self) -> Result<State, Error>;
    fn invalid_command(&self) -> Result<State, Error>;
}

impl Commands for Editor {
    fn append_command(&mut self, addr_1: usize) -> Result<State, Error> {
        self.addr = addr_1;
        self.insert_mode = true;
        Ok(State::Running)
    }

    fn insert_command(&mut self, addr_1: usize) -> Result<State, Error> {
        self.addr = addr_1;
        self.insert_mode = true;
        if self.addr > 0 {
            self.addr -= 1;
        }
        Ok(State::Running)
    }

    fn change_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error> {
        self.delete_command(addr_1, addr_2).ok();
        self.insert_mode = true;
        Ok(State::Running)
    }

    fn delete_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error> {
        self.lines.drain(addr_1 - 1 .. addr_2);
        self.addr = addr_1 - 1;
        self.dirty = true;
        Ok(State::Running)
    }

    fn edit_command(&mut self, params: Vec<&str>) -> Result<State, Error> {
        if params[0] == "" {
            return Err(Error::NoFilename);
        }
        let filename = params[0];
        self.filename = Some(filename.to_string());

        match read_lines(filename) {
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

    fn filename_command(&mut self, params: Vec<&str>) -> Result<State, Error> {
        if params[0] != "" {
            self.filename = Some(params[0].to_string());
        } else if let Some(f) = self.filename.clone() {
            println!("{}", f);
        } else {
            return Err(Error::NoFilename);
        }
        Ok(State::Running)
    }

    fn write_command(&mut self, params: Vec<&str>) -> Result<State, Error> {
        if params[0] != "" {
            self.filename = Some(params[0].to_string());
        }

        if let Some(f) = self.filename.clone() {
            let data = self.lines.join("\n");
            fs::write(f, data).expect("Unable to write file");
            self.dirty = false;
            Ok(State::Running)
        } else {
            return Err(Error::NoFilename);
        }
    }

    fn print_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error> {
        let n = self.lines.len();
        for i in addr_1 .. addr_2 + 1 {
            print_line(&self.lines[i - 1], i, n, false);
            self.addr = i;
        }
        Ok(State::Running)
    }

    fn number_command(&mut self, addr_1: usize, addr_2: usize) -> Result<State, Error> {
        let n = self.lines.len();
        for i in addr_1 .. addr_2 + 1 {
            print_line(&self.lines[i - 1], i, n, true);
            self.addr = i;
        }
        Ok(State::Running)
    }

    fn global_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> Result<State, Error> {
        let re = Regex::new(params[0]).unwrap();
        let cmd_list = if params.len() == 2 { params[1] } else { "p" };
        let show_number = cmd_list.ends_with("n");
        let mut i = addr_1;
        let mut n = addr_2;
        while i <= n {
            if re.is_match(&self.lines[i - 1]) {
                match cmd_list {
                    "p" | "pn" => {
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

    fn substitute_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> Result<State, Error> {
        let re = Regex::new(params[0]).unwrap();
        for i in addr_1 .. addr_2 + 1 {
            if re.is_match(&self.lines[i - 1]) {
                self.lines[i - 1] = re.replace_all(&self.lines[i - 1], params[1]).to_string();
                self.addr = i;
                self.dirty = true;
            }
        }
        Ok(State::Running)
    }

    fn quit_command(&self) -> Result<State, Error> {
        if self.dirty {
            Err(Error::Dirty)
        } else {
            Ok(State::Stopped)
        }
    }

    fn quit_without_checking_command(&self) -> Result<State, Error> {
        Ok(State::Stopped)
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
        ed.edit_command(vec![TEST_FILE]).ok();

        assert_eq!(ed.is_range_ok(0, 0, "a"), true);
        assert_eq!(ed.is_range_ok(1, 1, "a"), true);
        assert_eq!(ed.is_range_ok(TEST_FILE_LENGTH, TEST_FILE_LENGTH, "a"), true);
        assert_eq!(ed.is_range_ok(TEST_FILE_LENGTH + 1, TEST_FILE_LENGTH + 1,"a"), false);

        assert_eq!(ed.append_command(0), Ok(State::Running));
        assert_eq!(ed.addr, 0);
        assert_eq!(ed.insert_mode, true);

        assert_eq!(ed.append_command(1), Ok(State::Running));
        assert_eq!(ed.addr, 1);
        assert_eq!(ed.insert_mode, true);
    }
}
