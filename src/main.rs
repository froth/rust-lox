use std::{fs, io::{self, Write}};

use clap::Parser;

use crate::args::Args;

mod args;

fn main() {
    let args = Args::parse();
    match args.file {
        Some(file) => run_file(file).unwrap(),
        None => run_prompt().unwrap(),
    }
}

fn run_file(file: String) -> std::io::Result<()> {
     let contents = fs::read_to_string(file)?;
     run(contents);
     Ok(())
}

fn run_prompt() -> std::io::Result<()>  {
    let std = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut buf = String::new();
        match std.read_line(&mut buf)? {
            0 => return Ok(()),
            _ => run(buf),
        }
    }

}

//TODO: this should go in own module
fn run(source: String) {
    print!("{}", source)
}