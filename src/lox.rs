use miette::NamedSource;

use crate::{
    interpreter::Interpreter,
    parser::{parser_error::ParserError::ExpectedSemicolon, Parser},
    scanning::Scanner,
    value::Value,
};

#[derive(Default)]
pub struct Lox {
    interpreter: Interpreter,
    verbose: bool,
}

impl Lox {
    pub fn new(verbose: bool) -> Self {
        Self {
            interpreter: Interpreter::default(),
            verbose,
        }
    }

    pub fn run(&mut self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let tokens = Scanner::scan(source, named_source, self.verbose)?;
        let statements = Parser::parse(tokens, self.verbose)?;
        self.interpreter.interpret(statements)?;
        Ok(())
    }

    pub fn run_repl(&mut self, source: String) -> miette::Result<Option<Value>> {
        let named_source = NamedSource::new("repl", source.clone());
        let tokens = Scanner::scan(source, named_source, self.verbose)?;
        match Parser::parse(tokens, self.verbose) {
            Ok(statements) => {
                self.interpreter.interpret(statements)?;
                Ok(None)
            }
            Err(parser_errors) => match &parser_errors.parser_errors[..] {
                [ExpectedSemicolon {
                    expr: Some(expr),
                    src: _,
                    location: _,
                }] => {
                    if self.verbose {
                        eprintln!("No statement found. Fallback to expression:");
                        eprintln!("{}", expr);
                    }
                    let result = self.interpreter.interpret_expr(expr)?;
                    Ok(Some(result))
                }
                _ => Err(parser_errors)?,
            },
        }
    }
}

#[cfg(test)]
mod lox_tests {
    use crate::{
        interpreter::printer::{vec_printer::VecPrinter, Printer},
        interpreter::Interpreter,
    };
    use datadriven::walk;

    use super::Lox;
    impl Lox {
        pub fn with_printer(printer: Box<dyn Printer>) -> Self {
            Self {
                interpreter: Interpreter::new(printer),
                verbose: false,
            }
        }
    }
    #[test]
    fn integration_tests() {
        walk("tests/", |f| {
            f.run(|test_case| -> String {
                let input = test_case.input.to_string();
                let printer = VecPrinter::new();
                let mut lox = Lox::with_printer(Box::new(printer.clone()));
                lox.run_repl(input).unwrap();
                printer.get_output()
            })
        });
    }
}
