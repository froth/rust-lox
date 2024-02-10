use std::sync::Arc;

use miette::NamedSource;

use crate::{interpreting::Interpreter, parsing::Parser, scanning::Scanner, value::Value};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, source: String, named_source: NamedSource<String>) -> miette::Result<Value> {
        let named_source: Arc<NamedSource<String>> = named_source.into();
        let mut scanner = Scanner::new(source, named_source.clone());
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens);
        let expr = parser.parse()?;
        // println!("{}", expr);
        let value = expr.interpret()?;
        Ok(value)
    }
}
