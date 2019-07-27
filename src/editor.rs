use crate::addresses::*;
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
    Dirty
}

pub struct Editor {
    pub dirty: bool,
    pub show_help: bool,
    pub show_debug: bool,
    pub insert_mode: bool,
    pub addr: usize,
    pub lines: Vec<String>,
    pub filename: Option<String>
}

#[derive(Clone)]
pub struct CommandLine {
    pub addr_1: Option<usize>,
    pub addr_2: Option<usize>,
    pub cmd: String,
    pub flag: bool,
    pub params: Vec<String>
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
}
