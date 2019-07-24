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
}
