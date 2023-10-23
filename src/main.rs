use std::{
    env,
    error::Error,
    fmt,
    io::{self, BufRead, StdinLock},
    process::ExitCode,
};

use slug::slugify;

#[derive(Debug)]
struct SimpleError(String);

impl SimpleError {
    fn from_str(error: &str) -> Self {
        Self(String::from(error))
    }
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SimpleError {}

enum Operation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Csv,
}

impl TryFrom<&str> for Operation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "lowercase" {
            Ok(Operation::Lowercase)
        } else if value == "uppercase" {
            Ok(Operation::Uppercase)
        } else if value == "no-spaces" {
            Ok(Operation::NoSpaces)
        } else if value == "slugify" {
            Ok(Operation::Slugify)
        } else if value == "csv" {
            Ok(Operation::Csv)
        } else {
            Err(format!("Unrecognized operation: {}", value))
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowercase => write!(f, "lowercase"),
            Self::Uppercase => write!(f, "uppercase"),
            Self::NoSpaces => write!(f, "no-spaces"),
            Self::Slugify => write!(f, "slugify"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

struct Reader<'a> {
    input: StdinLock<'a>,
    buf: String,
}

impl<'a> Reader<'a> {
    fn stdin() -> Self {
        Self {
            input: io::stdin().lock(),
            buf: String::new(),
        }
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.clear();
        let Ok(len) = self.input.read_line(&mut self.buf) else {
            return None;
        };
        if len == 0 {
            return None;
        }
        Some(self.buf[..len - 1].to_string())
    }
}

struct Csv {
    columns: Vec<Column>,
}

impl Csv {
    fn from_reader(reader: &mut Reader) -> Result<Self, Box<dyn Error>> {
        let Some(header) = reader.next() else {
            return Err(Box::new(SimpleError::from_str("Empty CSV given.")));
        };

        let mut columns: Vec<Column> = header.split(',').map(Column::from_title).collect();
        if columns.is_empty() {
            return Err(Box::new(SimpleError::from_str("Empty header given.")));
        }

        for (i, line) in reader.enumerate() {
            let mut values: Vec<String> = line.split(',').map(String::from).collect();
            if values.len() != columns.len() {
                return Err(Box::new(SimpleError(format!(
                    "Line {}: invalid number of values.",
                    i + 2
                ))));
            }

            for (column, value) in columns.iter_mut().zip(values.drain(..)) {
                column.append(value);
            }
        }

        Ok(Self { columns })
    }

    fn width(&self) -> usize {
        let columns = self.columns.iter().map(|c| c.width).sum::<usize>();
        // padding (2 spaces) + |
        let extras = 3 * self.columns.len() + 1;
        columns + extras
    }

    fn hr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.width() {
            write!(f, "-")?;
        }
        writeln!(f)
    }

    fn row(&self, index: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for column in &self.columns {
            column.format(index, f)?;
        }
        writeln!(f, "|")
    }

    fn num_rows(&self) -> usize {
        self.columns[0].len()
    }
}

impl fmt::Display for Csv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.hr(f)?;
        self.row(0, f)?;
        self.hr(f)?;

        if self.num_rows() > 1 {
            for i in 1..self.num_rows() {
                self.row(i, f)?;
            }
            self.hr(f)?;
        }

        Ok(())
    }
}

struct Column {
    width: usize,
    values: Vec<String>,
}

impl Column {
    fn from_title(title: &str) -> Self {
        Self {
            width: title.chars().count(),
            values: vec![title.to_owned()],
        }
    }

    fn append(&mut self, value: String) {
        self.width = self.width.max(value.chars().count());
        self.values.push(value);
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn format(&self, index: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "| {:<width$} ", &self.values[index], width = self.width)
    }
}

fn main() -> ExitCode {
    let operation = match parse_args() {
        Ok(operation) => operation,
        Err(err) => {
            eprintln!("Arguments error: {}", err);
            return ExitCode::FAILURE;
        }
    };

    let reader = Reader::stdin();

    let result = match operation {
        Operation::Lowercase => lowercase(reader),
        Operation::Uppercase => uppercase(reader),
        Operation::NoSpaces => no_spaces(reader),
        Operation::Slugify => slugify_input(reader),
        Operation::Csv => csv(reader),
    };

    match result {
        Ok(output) => {
            print!("{}", output);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error while executing {}: {}", operation, error);
            ExitCode::FAILURE
        }
    }
}

fn parse_args() -> Result<Operation, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(format!(
            "Got incorrect number of arguments: {}.",
            args.len()
        ));
    }
    Operation::try_from(args[1].as_str())
}

fn lowercase(mut reader: Reader<'_>) -> Result<String, Box<dyn Error>> {
    match reader.next() {
        Some(line) => Ok(line.to_lowercase()),
        None => Err(Box::new(SimpleError::from_str("Empty input."))),
    }
}

fn uppercase(mut reader: Reader<'_>) -> Result<String, Box<dyn Error>> {
    match reader.next() {
        Some(line) => Ok(line.to_uppercase()),
        None => Err(Box::new(SimpleError::from_str("Empty input."))),
    }
}

fn no_spaces(mut reader: Reader<'_>) -> Result<String, Box<dyn Error>> {
    match reader.next() {
        Some(line) => Ok(line.replace(' ', "")),
        None => Err(Box::new(SimpleError::from_str("Empty input."))),
    }
}

fn slugify_input(mut reader: Reader<'_>) -> Result<String, Box<dyn Error>> {
    match reader.next() {
        Some(line) => Ok(slugify(line)),
        None => Err(Box::new(SimpleError::from_str("Empty input."))),
    }
}

fn csv(mut reader: Reader<'_>) -> Result<String, Box<dyn Error>> {
    Ok(Csv::from_reader(&mut reader)?.to_string())
}
