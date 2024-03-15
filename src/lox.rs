use miette::NamedSource;

use crate::{
    interpreter::{value::Value, Interpreter},
    parser::{parser_error::ParserError::ExpectedSemicolon, Parser},
    resolver::Resolver,
    scanning::Scanner,
};

pub struct Lox {
    interpreter: Interpreter,
    verbose: bool,
}

impl Lox {
    pub fn new(verbose: bool) -> Self {
        Self {
            interpreter: Interpreter::new(),
            verbose,
        }
    }

    pub fn run(&mut self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let tokens = Scanner::scan(source, named_source, self.verbose)?;
        let statements = Parser::parse(tokens, self.verbose)?;
        let locals = Resolver::resolve(&statements, self.verbose)?;
        self.interpreter.add_locals(locals);
        self.interpreter.interpret(&statements)?;
        Ok(())
    }

    pub fn run_repl(
        &mut self,
        source: String,
        repl_counter: usize,
    ) -> miette::Result<Option<Value>> {
        let named_source = NamedSource::new(format!("repl({repl_counter})"), source.clone());
        let tokens = Scanner::scan(source, named_source, self.verbose)?;
        match Parser::parse(tokens, self.verbose) {
            Ok(statements) => {
                let locals = Resolver::resolve(&statements, self.verbose)?;
                self.interpreter.add_locals(locals);
                self.interpreter.interpret(&statements)?;
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
    use miette::NamedSource;
    use serde_json::Value;

    use super::Lox;
    impl Lox {
        pub fn with_printer(printer: Box<dyn Printer>) -> Self {
            Self {
                interpreter: Interpreter::from_printer(printer),
                verbose: false,
            }
        }
    }
    #[test]
    fn integration_tests() {
        walk("tests/", |f| {
            let file_name = f.filename.clone();
            f.run(|test_case| -> String {
                let input = test_case.input.to_string();
                let printer = VecPrinter::new();
                let mut lox = Lox::with_printer(Box::new(printer.clone()));
                let named_source = NamedSource::new(file_name.clone(), input.clone());
                let result = lox.run(input, named_source);
                if test_case.directive == "error" {
                    let err = result.expect_err(
                        format!("Test {file_name} meant to be failing but succeeded").as_str(),
                    );
                    let handler = miette::JSONReportHandler::new();
                    let mut json = String::new();
                    handler.render_report(&mut json, err.as_ref()).unwrap();
                    format_json(json)
                } else {
                    result.unwrap_or_else(|_| {
                        panic!("Test {file_name} meant to be succeeding but failed.")
                    });
                    printer.get_output()
                }
            })
        });
    }

    fn format_json(json: String) -> String {
        let x: Value = serde_json::from_str(json.as_str()).unwrap();
        serde_json::to_string_pretty(&x).unwrap()
    }
}
