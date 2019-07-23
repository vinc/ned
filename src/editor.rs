use crate::utils::*;
use std::fs;
use regex::Regex;

static INVALID_COMMAND_ERROR: &str = "Invalid command";
static INVALID_RANGE_ERROR: &str = "Invalid range";
static NO_FILENAME_ERROR: &str = "No file name";
static DIRTY_ERROR: &str = "No write since last change";

pub struct Editor {
    pub dirty: bool,
    pub show_help: bool,
    pub show_debug: bool,
    pub insert_mode: bool,
    pub addr: usize,
    pub lines: Vec<String>,
    pub filename: Option<String>
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            dirty: false,
            show_debug: false,
            show_help: true,
            insert_mode: false,
            filename: None,
            lines: Vec::new(),
            addr: 0
        }
    }

    pub fn parse_addresses(&self, addr_1_str: &str, addr_sep_str: &str, addr_2_str: &str) -> (usize, usize) {
        let mut addr_1 = match addr_sep_str {
            "," | "%" => 1,
            _         => self.addr
        };

        addr_1 = match addr_1_str {
            ""  => addr_1,
            "." => self.addr,
            "$" => self.lines.len(),
            _   => addr_1_str.parse::<usize>().unwrap()
        };

        let mut addr_2 = match addr_sep_str {
            "," | "%" => self.lines.len(),
            _         => addr_1
        };

        addr_2 = match addr_2_str {
            ""  => addr_2,
            "." => self.addr,
            "$" => self.lines.len(),
            _   => addr_2_str.parse::<usize>().unwrap()
        };

        (addr_1, addr_2)
    }

    pub fn validate_addresses(&self, addr_1: usize, addr_2: usize, cmd: &str) -> bool {
        if addr_1 > addr_2 || addr_2 > self.lines.len() {
            if addr_1 != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                print_error(INVALID_RANGE_ERROR, self.show_help);
                return false;
            }
        }
        true
    }

    pub fn append_command(&mut self, addr_1: usize) -> bool {
        self.addr = addr_1;
        self.insert_mode = true;
        false
    }

    pub fn insert_command(&mut self, addr_1: usize) -> bool {
        self.addr = addr_1;
        self.insert_mode = true;
        if self.addr > 0 {
            self.addr -= 1;
        }
        false
    }

    pub fn change_command(&mut self, addr_1: usize, addr_2: usize) -> bool {
        self.delete_command(addr_1, addr_2);
        self.insert_mode = true;
        false
    }

    pub fn delete_command(&mut self, addr_1: usize, addr_2: usize) -> bool {
        self.lines.drain(addr_1 - 1 .. addr_2);
        self.addr = addr_1 - 1;
        self.dirty = true;
        false
    }

    pub fn edit_command(&mut self, params: Vec<&str>) -> bool {
        let filename = params[0];
        self.lines = read_lines(filename);
        self.addr = self.lines.len();
        self.filename = Some(filename.to_string());
        self.dirty = false;
        false
    }

    pub fn filename_command(&mut self, params: Vec<&str>) -> bool {
        if params[0] != "" {
            self.filename = Some(params[0].to_string());
        } else if let Some(f) = self.filename.clone() {
            println!("{}", f);
        } else {
            print_error(NO_FILENAME_ERROR, self.show_help);
        }
        false
    }

    pub fn write_command(&mut self, params: Vec<&str>) -> bool {
        if params[0] != "" {
            self.filename = Some(params[0].to_string());
        }

        if let Some(f) = self.filename.clone() {
            let data = self.lines.join("\n");
            fs::write(f, data).expect("Unable to write file");
            self.dirty = false;
            false
        } else {
            print_error(NO_FILENAME_ERROR, self.show_help);
            true
        }
    }

    pub fn print_command(&mut self, addr_1: usize, addr_2: usize) -> bool {
        let n = self.lines.len();
        for i in addr_1 .. addr_2 + 1 {
            print_line(&self.lines[i - 1], i, n, false);
            self.addr = i;
        }
        false
    }

    pub fn number_command(&mut self, addr_1: usize, addr_2: usize) -> bool {
        let n = self.lines.len();
        for i in addr_1 .. addr_2 + 1 {
            print_line(&self.lines[i - 1], i, n, true);
            self.addr = i;
        }
        false
    }

    pub fn global_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> bool {
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
        false
    }

    pub fn substitute_command(&mut self, addr_1: usize, addr_2: usize, params: Vec<&str>) -> bool {
        let re = Regex::new(params[0]).unwrap();
        for i in addr_1 .. addr_2 + 1 {
            if re.is_match(&self.lines[i - 1]) {
                self.lines[i - 1] = re.replace_all(&self.lines[i - 1], params[1]).to_string();
                self.addr = i;
                self.dirty = true;
            }
        }
        false
    }

    pub fn quit_command(&self) -> bool {
        if self.dirty {
            print_error(DIRTY_ERROR, self.show_help);
            false
        } else {
            true
        }
    }

    pub fn quit_without_checking_command(&self) -> bool {
        true
    }

    pub fn invalid_command(&self) -> bool {
        print_error(INVALID_COMMAND_ERROR, self.show_help);
        false
    }
}
