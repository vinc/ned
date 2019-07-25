use crate::editor::*;

pub trait Addresses {
    fn parse_addresses(&self, addr_1_str: &str, addr_sep_str: &str, addr_2_str: &str) -> (usize, usize);
    fn is_range_ok(&self, addr_1: usize, addr_2: usize, cmd: &str) -> bool;
}

impl Addresses for Editor {
    fn parse_addresses(&self, addr_1_str: &str, addr_sep_str: &str, addr_2_str: &str) -> (usize, usize) {
        let addr_1 = match addr_1_str {
            ""  => if addr_sep_str == "" { self.addr } else { 1 }
            "." => self.addr,
            "$" => self.lines.len(),
            _   => addr_1_str.parse::<usize>().unwrap()
        };

        let addr_2 = match addr_2_str {
            ""  => if addr_sep_str == "" { addr_1 } else { self.lines.len() }
            "." => self.addr,
            "$" => self.lines.len(),
            _   => addr_2_str.parse::<usize>().unwrap()
        };

        (addr_1, addr_2)
    }

    fn is_range_ok(&self, addr_1: usize, addr_2: usize, cmd: &str) -> bool {
        if addr_1 > addr_2 || addr_2 > self.lines.len() {
            if addr_1 != 0 || cmd != "a" || !cmd.to_lowercase().ends_with("q") {
                return false;
            }
        }
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
    fn test_parse_addresses() {
        let mut ed = Editor::new();
        ed.edit_command(vec![TEST_FILE]).ok();

        assert_eq!(ed.addr, TEST_FILE_LENGTH);
        assert_eq!(ed.parse_addresses("", "%", ""), (1, TEST_FILE_LENGTH));
        assert_eq!(ed.parse_addresses("2", "", ""), (2, 2));
    }
}
