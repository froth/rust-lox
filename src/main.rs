use clap::Parser;
use lox::Lox;

use args::Args;
use expr::{Expr, Literal};
use token::{Token, TokenType};

mod args;
mod error_reporter;
mod expr;
mod lox;
mod scanner;
mod token;

fn main() {
    let args = Args::parse();
    let mut lox = Lox::default();
    //TODO: remove
    let expr = Expr::Binary(
        Box::new(Expr::Unary(
            Token::new(TokenType::Minus, "-", 1),
            Box::new(Expr::Literal(Literal::Number(123.0))),
        )),
        Token::new(TokenType::Star, "*", 1),
        Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::String(
            "45.67".to_string(),
        ))))),
    );
    println!("{}", expr);
    match args.file {
        Some(file) => {
            if !lox.run_file(file) {
                std::process::exit(65)
            }
        }
        None => lox.run_prompt(),
    }
}
