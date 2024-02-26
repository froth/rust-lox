use std::fs;

use clap::Parser;
use lox::Lox;

use args::Args;
use miette::{IntoDiagnostic, NamedSource};
use rustyline::{
    error::ReadlineError, highlight::MatchingBracketHighlighter,
    validate::MatchingBracketValidator, Completer, Editor, Helper, Highlighter, Hinter, Validator,
};

mod args;
mod ast;
mod interpreter;
mod lox;
mod parsing;
mod printer;
mod scanning;
mod source_span_extensions;
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
    let mut lox = Lox::default();
    lox.run(contents, named_source)
}

#[derive(Helper, Completer, Hinter, Validator, Highlighter, Default)]
struct MyHelper {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
}

fn run_prompt() -> miette::Result<()> {
    const HISTORY_FILE: &str = ".history";
    let mut rl = Editor::new().into_diagnostic()?;
    rl.set_helper(Some(MyHelper::default()));
    rl.load_history(HISTORY_FILE)
        .unwrap_or_else(|_err| println!("No history file found: {}", HISTORY_FILE));
    let mut lox = Lox::default();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(source) => {
                rl.add_history_entry(source.as_str()).into_diagnostic()?;
                lox.run_stdin(source)
                    .unwrap_or_else(|err| eprintln!("{:?}", err));
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            err => {
                err.into_diagnostic()?;
            }
        }
    }
    rl.save_history(HISTORY_FILE).into_diagnostic()?;
    Ok(())
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
