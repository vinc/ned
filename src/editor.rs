pub enum State {
    Running,
    Stopped
}

pub enum Error {
    InvalidCommand,
    InvalidAddress,
    NoFilename,
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

    pub fn is_range_ok(&self, addr_1: usize, addr_2: usize, cmd: &str) -> bool {
        if addr_1 > addr_2 || addr_2 > self.lines.len() {
            if addr_1 != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                return false;
            }
        }
        true
    }
}
