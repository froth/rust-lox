use std::{
    fs,
    io::{self, Write},
};

use clap::Parser;
use lox::Lox;

use args::Args;
use miette::{IntoDiagnostic, NamedSource};
use printer::ConsolePrinter;

mod args;
mod ast;
mod interpreter;
mod lox;
mod parsing;
mod printer;
mod scanning;
mod source_span_extensions;
mod token;
mod types;
mod value;

fn main() {
    let args = Args::parse();
    let result = match args.file {
        Some(file) => run_file(file),
        None => run_prompt(),
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(65)
        }
    };
}

fn run_file(file: String) -> miette::Result<()> {
    let contents = fs::read_to_string(file.clone()).into_diagnostic()?;

    let named_source = NamedSource::new(file, contents.clone());
    let lox = Lox::new(&ConsolePrinter);
    lox.run(contents, named_source)
}

fn run_prompt() -> miette::Result<()> {
    let std = io::stdin();
    let lox = Lox::new(&ConsolePrinter);
    loop {
        print!("> ");
        io::stdout().flush().into_diagnostic()?;
        let mut buf = String::new();
        match std.read_line(&mut buf).into_diagnostic()? {
            0 => return Ok(()),
            _ => {
                let source = buf.trim_end().to_string();
                lox.run_stdin(source)
                    .unwrap_or_else(|err| eprintln!("{:?}", err));
            }
        }
    }
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
