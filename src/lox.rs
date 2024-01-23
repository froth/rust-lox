use std::{
    fs,
    io::{self, Write},
};

use miette::IntoDiagnostic;

use crate::{parser::Parser, scanner::Scanner};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    fn run(&mut self, source: String, filename: String) -> miette::Result<()> {
        let mut scanner = Scanner::new(source, filename);
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens);
        println!("{}", parser.parse());
        Ok(())
    }
    pub fn run_file(&mut self, file: String) -> miette::Result<()> {
        let contents = fs::read_to_string(file.clone()).into_diagnostic()?;
        self.run(contents, file)?;
        Ok(())
    }

    pub fn run_prompt(&mut self) -> miette::Result<()> {
        let std = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().into_diagnostic()?;
            let mut buf = String::new();
            match std.read_line(&mut buf).into_diagnostic()? {
                0 => return Ok(()),
                _ => match self.run(buf.trim_end().to_string(), "stdin".to_string()) {
                    Ok(_) => (),
                    Err(err) => println!("{:?}", err),
                },
            }
        }
    }
}
