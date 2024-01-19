use clap::Parser;
use lox::Lox;

use args::Args;

mod args;
mod error;
mod error_reporter;
mod expr;
mod lox;
mod parser;
mod scanner;
mod token;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::new();
    match args.file {
        Some(file) => {
            if !lox.run_file(file) {
                std::process::exit(65)
            }
        }
        None => lox.run_prompt(),
    }
}

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
