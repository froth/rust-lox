use std::{
    fs,
    io::{self, Write},
};


#[derive(Default)]
pub struct Lox {
    error_reporter: ErrorReporter
}

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug)]
enum TokenType {
    //single character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual, Equal, EqualEqual, Greater, GreaterEqual, Less, LessEqual,
    // Eof
    Eof
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, line: usize) -> Self { Self { token_type, lexeme, line } }
}

impl Scanner {
    fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_token(&mut self, char: char, error_reporter: &mut ErrorReporter) {
        use TokenType::*;
        match char {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            _ => error_reporter.error(self.line, "Unexpected character.")
        };
    }

    fn scan_tokens(&mut self, error_reporter: &mut ErrorReporter) -> &Vec<Token> {
        while let Some(char) = self.advance() {
            self.start = self.current -1; //has already been advanced
            self.scan_token(char, error_reporter)
        }

        self.tokens.push(Token::new(TokenType::Eof, String::new(), self.line));
        &self.tokens
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line))
    }

    fn advance(&mut self) -> Option<char> {
        let char = self.source.chars().nth(self.current);
        if char.is_some() {
            self.current += 1;
        }
        char
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.source.chars().nth(self.current) {
            Some(x) if x == expected => {
                self.current += 1;
                true
            },
            _ => false
        }

    }
}

#[derive(Default)]
struct ErrorReporter{
    had_error: bool,
}

impl ErrorReporter {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, place, message);
        self.had_error = true;
    }

    pub fn reset(&mut self) {
        self.had_error = false
    }
}

impl Lox {
    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens(&mut self.error_reporter);
        tokens.iter().for_each(|x| println!("{:?}", x))
    }
    pub fn run_file(&mut self, file: String) {
        let contents = fs::read_to_string(file).unwrap();
        self.run(contents);
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
