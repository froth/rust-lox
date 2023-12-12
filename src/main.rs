use clap::Parser;
use lox::Lox;

use args::Args;

mod args;
mod error_reporter;
mod lox;
mod scanner;
mod token;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::default();
    match args.file {
        Some(file) => {
            if !lox.run_file(file) {
                std::process::exit(65)
            }
        }
        None => lox.run_prompt(),
    }
}
