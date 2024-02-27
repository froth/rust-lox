use miette::NamedSource;

use crate::{
    interpreter::Interpreter,
    parsing::{parser_error::ParserError::ExpectedSemicolon, Parser},
    scanning::Scanner,
    value::Value,
};

#[derive(Default)]
pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn run(&mut self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let tokens = Scanner::scan(source, named_source)?;
        let statements = Parser::parse(tokens)?;
        self.interpreter.interpret(statements)?;
        Ok(())
    }

    pub fn run_repl(&mut self, source: String) -> miette::Result<Option<Value>> {
        let named_source = NamedSource::new("repl", source.clone());
        let tokens = Scanner::scan(source, named_source)?;
        match Parser::parse(tokens) {
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
    //TODO: unpub printer once these tests are migrated to file based

    use crate::{
        interpreter::printer::{vec_printer::VecPrinter, Printer},
        interpreter::Interpreter,
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
        let mut lox = Lox::new(Box::new(printer.clone()));
        lox.run_repl(r#"print "string";"#.to_string()).unwrap();
        assert_eq!(printer.get_lines(), vec!["string".into()])
    }

    #[test]
    fn print_variable() {
        let printer = VecPrinter::new();
        let mut lox = Lox::new(Box::new(printer.clone()));
        lox.run_repl(
            r#"
            var x = "string";
            print "x=" + x;
        "#
            .to_string(),
        )
        .unwrap();
        assert_eq!(printer.get_lines(), vec!["x=string".into()])
    }
}
