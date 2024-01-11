use std::{
    fs,
    io::{self, Write},
};

use crate::{
    error_reporter::{ConsoleErrorReporter, ErrorReporter},
    parser::Parser,
    scanner::Scanner,
};

pub struct Lox {
    error_reporter: Box<dyn ErrorReporter>,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            error_reporter: Box::new(ConsoleErrorReporter::new()),
        }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source, self.error_reporter.as_mut());
        let tokens = scanner.scan_tokens();
        // tokens.iter().for_each(|x| println!("{:?}", x));
        if !self.error_reporter.had_error() {
            let mut parser = Parser::new(tokens, self.error_reporter.as_mut());
            println!("{}", parser.parse());
        }
    }
    pub fn run_file(&mut self, file: String) -> bool {
        let contents = fs::read_to_string(file).unwrap();
        self.run(contents);
        self.error_reporter.had_error() // TODO: better error handling then boolean...
    }

    pub fn run_prompt(&mut self) {
        let std = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buf = String::new();
            match std.read_line(&mut buf).unwrap() {
                0 => return,
                _ => {
                    self.run(buf.trim_end().to_string());
                    self.error_reporter.reset();
                }
            }
        }
    }
}
