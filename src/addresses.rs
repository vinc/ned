use crate::editor::*;
use regex::Regex;

pub trait Addresses {
    fn parse_addr(&self, addr: &str) -> Option<usize>;
    fn parse_addr_1(&self, addr: &str, sep: &str) -> Option<usize>;
    fn parse_addr_2(&self, addr: &str, sep: &str) -> Option<usize>;
    fn search_forward_regex(&self, pattern: &str) -> Option<usize>;
    fn is_range_ok(&self, cl: CommandLine) -> bool;
}

impl Addresses for Editor {
    fn parse_addr(&self, addr: &str) -> Option<usize> {
        match &addr[0..1] {
            "." => Some(self.addr),
            "$" => Some(self.lines.len()),
            "/" => self.search_forward_regex(&addr[1..addr.len() - 1]),
            "+" => Some(self.addr + addr[1..].parse::<usize>().unwrap()),
            "-" => Some(self.addr - addr[1..].parse::<usize>().unwrap()),
            _   => Some(addr.parse::<usize>().unwrap()),
        }
    }

    fn parse_addr_1(&self, addr: &str, sep: &str) -> Option<usize> {
        if addr == "" {
            if sep == "" { None } else { Some(1) }
        } else {
            self.parse_addr(addr)
        }
    }

    fn parse_addr_2(&self, addr: &str, sep: &str) -> Option<usize> {
        if addr == "" {
            if sep == "" { None } else { Some(self.lines.len()) }
        } else {
            self.parse_addr(addr)
        }
    }

    fn search_forward_regex(&self, pattern: &str) -> Option<usize> {
        let re = Regex::new(pattern).unwrap();
        let n = self.lines.len();
        let mut i = self.addr;
        loop {
            if i == n { // Wrap around to the beginning
                i = 1;
            } else {
                i += 1;
            }
            if re.is_match(&self.lines[i - 1]) { // Return first match
                return Some(i);
            } else if i == self.addr { // No matches in all the buffer
                return None;
            }
        }
    }

    fn is_range_ok(&self, cl: CommandLine) -> bool {
        // FIXME: Validate range
        /*
        if addr_1 > addr_2 || addr_2 > self.lines.len() {
            if addr_1 != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                return false;
            }
        }
        */
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::*;

    static TEST_FILE: &str = "LICENSE";
    static TEST_FILE_LENGTH: usize = 21;

    #[test]
    fn test_parse_addr() {
        let mut ed = Editor::new();
        let cl = CommandLine {
            addr_1: None,
            addr_2: None,
            cmd: "e".to_string(),
            flag: false,
            params: vec![TEST_FILE.to_string()]
        };
        ed.edit_command(cl).ok();
        ed.addr = 10;
        assert_eq!(ed.parse_addr("2"), Some(2));
        assert_eq!(ed.parse_addr("."), Some(10));
        assert_eq!(ed.parse_addr("$"), Some(TEST_FILE_LENGTH));
        assert_eq!(ed.parse_addr("+2"), Some(12));
        assert_eq!(ed.parse_addr("-2"), Some(8));
        assert_eq!(ed.parse_addr("/free/"), Some(5));
    }
}
