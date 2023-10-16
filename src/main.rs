use std::{
    env,
    io::{self, BufRead},
};

use slug::slugify;

enum Operation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
}

impl TryFrom<&str> for Operation {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "lowercase" {
            Ok(Operation::Lowercase)
        } else if value == "uppercase" {
            Ok(Operation::Uppercase)
        } else if value == "no-spaces" {
            Ok(Operation::NoSpaces)
        } else if value == "slugify" {
            Ok(Operation::Slugify)
        } else {
            Err("Unrecognized operation.")
        }
    }
}

fn main() {
    let operation = parse_args();

    let mut buf = String::new();
    let mut stdin = io::stdin().lock();

    while let Ok(len) = stdin.read_line(&mut buf) {
        let line = &buf[..len];
        let transformed = match operation {
            Operation::Lowercase => line.to_lowercase(),
            Operation::Uppercase => line.to_uppercase(),
            Operation::NoSpaces => line.replace(' ', ""),
            Operation::Slugify => slugify(line),
        };
        buf.clear();

        print!("{}", transformed);
    }
}

fn parse_args() -> Operation {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Got incorrect number of arguments: {}.", args.len());
    }
    Operation::try_from(args[1].as_str()).unwrap()
}
