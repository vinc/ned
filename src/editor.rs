use crate::addresses::*;
use crate::commands::*;
use crate::utils::*;
use std::fs;
use regex::Regex;

static RE_ADDRS: &str =
    r"^(?P<addr1>/(?:[^\\/]|\\.)*/|[.$]|[-+]?[0-9]*)(?P<sep>[,;%]?)(?P<addr2>/(?:[^\\/]|\\.)*/|[.$]|[-+]?[0-9]*)";
static RE_CMD: &str =
    r"^(?P<cmd>[a-z]*)(?P<flag>!?)(?:/(?P<re1>(?:[^\\/]|\\.)*)/(?P<re2>(?:[^\\/]|\\.)*)?)?(?:[ /](?P<params>.*))?";

#[derive(Debug, PartialEq)]
pub enum State {
    Running,
    Stopped
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidCommand,
    InvalidAddress,
    NoFilename,
    CannotOpen,
    NoUndo,
    Dirty
}

#[derive(Clone, Debug)]
pub struct Editor {
    pub dirty: bool,
    pub show_help: bool,
    pub show_debug: bool,
    pub insert_mode: bool,
    pub filename: Option<String>,
    pub addr: usize,
    pub lines: Vec<String>,
    pub history: Vec<String>
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            dirty: false,
            show_debug: false,
            show_help: true,
            insert_mode: false,
            filename: None,
            addr: 0,
            lines: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn parse_command_line(&self, input: &str) -> CommandLine {
        let re = Regex::new(RE_ADDRS).unwrap();
        let caps = re.captures(input).unwrap();

        let addr_1 = self.parse_addr_1(&caps["addr1"], &caps["sep"]);
        let addr_2 = self.parse_addr_2(&caps["addr2"], &caps["sep"]);

        let i = caps[0].len();
        let re = Regex::new(RE_CMD).unwrap();
        let caps = re.captures(&input[i..]).unwrap();

        let cmd = caps["cmd"].to_string();
        let flag = &caps["flag"] == "!";

        let mut params = vec![];
        if let Some(m) = caps.name("re1") {
            params.push(m.as_str().to_string());
        }
        if let Some(m) = caps.name("re2") {
            params.push(m.as_str().to_string());
        }
        if let Some(m) = caps.name("params") {
            for param in m.as_str().split_whitespace() {
                params.push(param.to_string());
            }
        }

        CommandLine { addr_1, addr_2, cmd, flag, params }
    }

    pub fn exec(&mut self, input: &str) -> Result<State, Error> {
        if self.insert_mode {
            if input == "." {
                self.insert_mode = false;
            } else {
                self.lines.insert(self.addr, input.to_string());
                self.addr += 1;
                self.dirty = true;
            }
            self.log(input);
            return Ok(State::Running);
        }

        let mut cl = self.parse_command_line(input);

        if !self.is_range_ok(cl.clone()) {
            return Err(Error::InvalidAddress);
        }
        let mut loggable_input = input.to_string();
        if cl.addr_1.is_none() {
            loggable_input = format!("{}{}", self.addr, input);
            cl.addr_1 = Some(self.addr);
        }
        if cl.addr_2.is_none() {
            cl.addr_2 = cl.addr_1;
        }

        if self.show_debug {
            println!("# {:?}", self);
        }

        if cl.is_undo() {
            self.undo()
        } else {
            if cl.is_undoable() {
                self.log(&loggable_input);
            }
            self.command(cl)
        }
    }

    pub fn log(&mut self, input: &str) {
        self.history.push(input.to_string());
        self.mk_log();
    }

    pub fn mk_log(&mut self) {
        if let Some(filename) = self.filename.clone() {
            let file = format!(".{}.ned", filename);
            let data = self.history.join("\n") + "\n";
            fs::write(file, data).expect("Unable to write ned file");
        }
    }

    pub fn rm_log(&self) {
        if let Some(filename) = self.filename.clone() {
            let file = format!(".{}.ned", filename);
            fs::remove_file(file).expect("Unable to remove ned file");
        }
    }

    pub fn mk_tmp(&self) {
        if let Some(filename) = self.filename.clone() {
            let file = format!(".{}.tmp", filename);
            let data = self.lines.join("\n") + "\n";
            fs::write(file, data).expect("Unable to write tmp file");
        }
    }

    pub fn rm_tmp(&self) {
        if let Some(filename) = self.filename.clone() {
            let file = format!(".{}.tmp", filename);
            fs::remove_file(file).expect("Unable to remove tmp file");
        }
    }

    pub fn undo(&mut self) -> Result<State, Error> {
        if let Some(_) = self.history.pop() {
            let inputs = self.history.clone();
            if let Some(filename) = self.filename.clone() {
                let tmp = format!(".{}.tmp", filename);
                if let Ok(lines) = read_lines(&tmp) {
                    self.lines = lines; // Restore backup
                    self.addr = self.lines.len();
                    self.dirty = false;
                    for input in inputs {
                        self.exec(&input).ok();
                    }
                }
            }
            self.mk_log();
            Ok(State::Running)
        } else {
            Err(Error::NoUndo)
        }
    }
}
