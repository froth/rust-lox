use std::sync::Arc;

use miette::NamedSource;

use crate::{interpreter::Interpreter, parsing::Parser, printer::Printer, scanning::Scanner};

pub struct Lox<'a> {
    printer: &'a dyn Printer,
}

impl<'a> Lox<'a> {
    pub fn new(printer: &'a impl Printer) -> Self {
        Self { printer }
    }

    pub fn run(&self, source: String, named_source: NamedSource<String>) -> miette::Result<()> {
        let named_source: Arc<NamedSource<String>> = named_source.into();
        let mut scanner = Scanner::new(source, named_source.clone());
        let tokens = scanner.scan_tokens()?;
        // tokens.iter().for_each(|x| println!("{:?}", x));
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        // statements.iter().for_each(|stmt| print!("{}", stmt));
        let interpreter = Interpreter::new(self.printer);
        interpreter.interpret(statements)?;
        Ok(())
    }

    pub fn run_stdin(&self, source: String) -> miette::Result<()> {
        let named_source: NamedSource<String> = NamedSource::new("stdin", source.clone());
        self.run(source, named_source)
    }
}

#[cfg(test)]
mod lox_tests {

    use crate::printer::vec_printer::VecPrinter;

    use super::Lox;

    #[test]
    fn print_string_literal() {
        let printer = VecPrinter::new();
        let lox = Lox::new(&printer);
        lox.run_stdin(r#"print "string";"#.to_string()).unwrap();
        assert_eq!(printer.get_lines(), vec!["string".to_string().into()])
    }
}
