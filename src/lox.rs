use std::sync::Arc;

use miette::NamedSource;

use crate::{parsing::Parser, scanning::Scanner};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        source: String,
        named_source: NamedSource<String>,
    ) -> miette::Result<()> {
        let named_source: Arc<NamedSource<String>> = named_source.into();
        let mut scanner = Scanner::new(source, named_source.clone());
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens, named_source.clone());
        println!("{}", parser.parse()?);
        Ok(())
    }
}
