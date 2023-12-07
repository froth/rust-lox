
use clap::Parser;
use lox::Lox;

use args::Args;

mod args;
mod lox;
mod scanner;
mod error_reporter;
mod token;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::default();
    match args.file {
        Some(file) => lox.run_file(file),
        None => lox.run_prompt(),
    }
}

