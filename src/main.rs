extern crate colored;
extern crate regex;

use colored::Colorize;
use regex::Regex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io;

fn read(path: &str) -> Vec<String> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    data.lines().map(|l| l.to_string()).collect()
}

fn main() {
    let mut insert_mode = false;
    let mut file = None;
    let mut addr = 0;
    let mut text = Vec::new();

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        file = Some(args[1].clone());
        text = read(&args[1]);
        addr = text.len();
    }

    let re = Regex::new(r"(?P<addr1>\d*)(?P<sep>[,;]?)(?P<addr2>\d*)(?P<cmd>[a-z]*) ?(?P<params>\w*)").unwrap();

    loop {
        let mut input = String::new();
        if !insert_mode {
            print!(":");
        }
        io::stdout().flush().ok();
        io::stdin().read_line(&mut input).expect("Unable to read stdin");

        if insert_mode {
            if input.trim() == "." {
                insert_mode = false;
            } else {
                let line = String::from(input.trim());
                text.insert(addr, line);
                addr += 1;
            }
            continue;
        }


        let caps = re.captures(input.trim()).unwrap();
        println!("# regex: {:?}", caps);

        let cmd = &caps["cmd"];

        if cmd == "q" {
            break;
        }

        let params: Vec<&str> = caps["params"].split_whitespace().collect();

        // Range
        let mut begin = addr;
        let mut end = addr;

        if caps["addr1"].len() > 0 {
            begin = caps["addr1"].parse::<usize>().unwrap();
            end = begin;
        } else if &caps["sep"] == "," {
            begin = 1;
            end = text.len();
        }
        if caps["addr2"].len() > 0 {
            end = caps["addr2"].parse::<usize>().unwrap();
        }
        addr = end;

        if begin > end || end > text.len() {
            println!("?");
            continue;
        }

        let range = begin .. end + 1;
        println!("# range: [{},{}]", begin, end);
        println!("# addr: {}", addr);
        println!("# cmd: {}", cmd);
        println!("# params: {:?}", params);

        match cmd {
            "" => {
                println!("{}", text[addr - 1]);
            },
            "a" => { // Add line after addr
                insert_mode = true;
            },
            "i" => { // Insert line before addr
                insert_mode = true;
                if addr > 0 {
                    addr -= 1;
                }
            },
            "d" => {
                text.remove(addr - 1);
            },
            "o" => {
                let f = params[1];
                file = Some(String::from(f));
                text = read(params[1]);
            },
            "w" => {
                let path = if params.len() == 2 {
                    String::from(params[1])
                } else if let Some(f) = file.clone() {
                    f
                } else {
                    println!("No file name");
                    continue;
                };
                let data = text.join("\n");
                fs::write(path, data).expect("Unable to write file");
            },
            "p" => {
                for i in range {
                    println!("{}", &text[i - 1]);
                }
            },
            "n" | "pn" => {
                let width = text.len().to_string().len();
                for i in range {
                    let num = format!("{:>width$}", i, width = width).green();
                    let cur = if i == addr {
                        "*".green()
                    } else {
                        " ".normal()
                    };
                    println!("{}{} {}", cur, num, &text[i - 1]);
                }
            },
            _ => {
                println!("?");
            }
        }
    }
}
