use std::{fs, io::{self, Write}};

#[derive(Default)]
pub struct Lox {
    had_error: bool
}

struct Scanner {
    source: String
}

#[derive(Debug)]
enum Token {

}

impl Scanner {
    fn new(source: String) -> Self { Self { source } }

    fn scan_tokens(&self) -> Vec<Token>{
        vec![]
    }
}

impl Lox {
    fn run(&self, source: String) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        tokens.iter().for_each(|x| println!("{:?}", x))
    }
    pub fn run_file(&self, file: String) {
        let contents = fs::read_to_string(file).unwrap();
        self.run(contents);
    }

    pub fn run_prompt(&mut self)  {
        let std = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buf = String::new();
            match std.read_line(&mut buf).unwrap() {
                0 => return,
                _ => {
                    self.run(buf);
                    self.had_error = false
                },
            }
        }

}

    fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message)
    }

    fn report(&mut self, line: i32, place: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, place, message);
        self.had_error = true;
    }
}