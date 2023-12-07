use std::{fs, io::{self, Write}};

use clap::Parser;
use lox::Lox;

use args::Args;

mod args;
mod lox;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::default();
    match args.file {
        Some(file) => lox.run_file(file),
        None => lox.run_prompt(),
    }
}

