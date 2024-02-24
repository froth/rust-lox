
use std::sync::Arc;

use miette::NamedSource;

use crate::{
    interpreter::Interpreter, parsing::Parser, printer::ConsolePrinter, scanning::Scanner,
};

pub struct Lox {
    interpreter: Interpreter,
}

impl Default for Lox {
    fn default() -> Self {
        Self {
            interpreter: Interpreter::new(Box::new(ConsolePrinter)),
        }
    }
}

impl Lox {
    pub fn run(&self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let named_source: Arc<NamedSource<String>> = named_source.into();
        let mut scanner = Scanner::new(source, named_source.clone());
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        // statements
        //     .iter()
        //     .for_each(|stmt| print!("{} -> {:?}", stmt, stmt.location));
        self.interpreter.interpret(statements)?;
        Ok(())
    }

    pub fn run_stdin(&self, source: String) -> miette::Result<()> {
        let named_source: NamedSource<String> = NamedSource::new("stdin", source.clone());
        self.run(source, named_source)
    }
}

#[cfg(test)]
mod lox_tests {

    use crate::{
        interpreter::Interpreter,
        printer::{vec_printer::VecPrinter, Printer},
    };

    use super::Lox;
    impl Lox {
        pub fn new(printer: Box<dyn Printer>) -> Self {
            Self {
                interpreter: Interpreter::new(printer),
            }
        }
    }

    #[test]
    fn print_string_literal() {
        let printer = VecPrinter::new();
        let lox = Lox::new(Box::new(printer.clone()));
        lox.run_stdin(r#"print "string";"#.to_string()).unwrap();
        assert_eq!(printer.get_lines(), vec!["string".to_string().into()])
    }
}
