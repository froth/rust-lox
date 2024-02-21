use std::sync::Arc;

use miette::NamedSource;

use crate::{interpreting::Interpreter, parsing::Parser, scanning::Scanner};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let named_source: Arc<NamedSource<String>> = named_source.into();
        let mut scanner = Scanner::new(source, named_source.clone());
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        statements.iter().for_each(|stmt| print!("{}", stmt));
        statements.iter().try_for_each(|stmt| stmt.interpret())?;
        Ok(())
    }
}
