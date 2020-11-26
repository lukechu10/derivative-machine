#![feature(box_patterns)]

use colored::Colorize;
use logos::Logos;
use parser::{ExprVisitor, Parser};
use passes::simplify::SimplifyVisitor;
use std::error::Error;
use std::io::Write;

mod lexer;
mod parser;
mod passes;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("{} ", ">".bright_black());
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        let mut tokens = lexer::Token::lexer(&line);
        let mut parser = Parser::from(&mut tokens);
        let mut ast = parser.parse();
        if !parser.errors().is_empty() {
            dbg!(parser.errors());
        }

        let mut fold_visitor = SimplifyVisitor;
        fold_visitor.visit(&mut ast);
        println!("{} {}", "<".bright_black(), format!("{}", ast).yellow());
    }
}
