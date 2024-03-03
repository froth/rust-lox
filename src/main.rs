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
mod scanning;
mod source_span_extensions;
mod types;
mod value;

fn main() {
    let args = Args::parse();
    let result = match args.file {
        Some(file) => run_file(file),
        None => run_prompt(args).into_diagnostic(),
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

fn run_prompt(args: Args) -> rustyline::Result<()> {
    let history_file = args.history_file;
    let mut rl = Editor::new()?;
    rl.set_helper(Some(MyHelper::default()));
    if let Err(err) = rl.load_history(&history_file).into_diagnostic() {
        eprintln!("No previous history: {:?}", &history_file);
        if args.verbose {
            eprintln!("Error: {:?}", err)
        }
    }
    let mut lox = Lox::default();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(source) => {
                rl.add_history_entry(source.as_str())?;
                match lox.run_repl(source) {
                    Ok(Some(value)) => println!("expr => {}", value),
                    Ok(None) => (),
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            err => {
                err?;
            }
        }
    }

    if let Err(err) = rl.save_history(&history_file).into_diagnostic() {
        eprintln!("Unable to save history: {:?}", err);
    }
    Ok(())
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
