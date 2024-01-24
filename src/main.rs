use clap::Parser;
use lox::Lox;

use args::Args;

mod args;
mod expr;
mod lox;
mod parser;
mod scanner;
mod token;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::new();
    let result = match args.file {
        Some(file) => lox.run_file(file),
        None => lox.run_prompt(),
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(65)
        }
    };
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
