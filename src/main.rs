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
mod parser;
mod resolver;
mod scanner;
mod source_span_extensions;

fn main() {
    let args = Args::parse();
    let lox = Lox::new(args.verbose);
    let result = match args.file {
        Some(file) => run_file(lox, file),
        None => run_prompt(lox, args).into_diagnostic(),
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(65)
        }
    };
}

fn run_file(mut lox: Lox, file: String) -> miette::Result<()> {
    let contents = fs::read_to_string(file.clone()).into_diagnostic()?;

    let named_source = NamedSource::new(file, contents.clone());
    lox.run(contents, named_source)
}

fn run_prompt(mut lox: Lox, args: Args) -> rustyline::Result<()> {
    #[derive(Helper, Completer, Hinter, Validator, Highlighter, Default)]
    struct MyHelper {
        #[rustyline(Validator)]
        validator: MatchingBracketValidator,
        #[rustyline(Highlighter)]
        highlighter: MatchingBracketHighlighter,
    }

    let history_file = args.history_file;
    let mut rl = Editor::new()?;
    rl.set_helper(Some(MyHelper::default()));
    if let Err(err) = rl.load_history(&history_file).into_diagnostic() {
        eprintln!("No previous history: {:?}", &history_file);
        if args.verbose {
            eprintln!("Error: {:?}", err)
        }
    }
    let mut repl_counter: usize = 0;
    loop {
        repl_counter += 1;
        let readline = rl.readline(">> ");
        match readline {
            Ok(source) => {
                rl.add_history_entry(source.as_str())?;
                match lox.run_repl(source, repl_counter) {
                    Ok(Some(value)) => println!("expr => {}", value),
                    Ok(None) => (),
                    Err(err) => println!("{:?}", err),
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
