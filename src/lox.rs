use std::{
    fs,
    io::{self, Write},
};

use crate::{error_reporter::ErrorReporter, scanner::Scanner};


#[derive(Default)]
pub struct Lox {
    error_reporter: ErrorReporter
}



impl Lox {
    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source, & mut self.error_reporter);
        let tokens = scanner.scan_tokens();
        tokens.iter().for_each(|x| println!("{:?}", x))
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
