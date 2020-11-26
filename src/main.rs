use logos::Logos;
use parser::Parser;
use std::error::Error;
use std::io::Write;

mod lexer;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        let mut tokens = lexer::Token::lexer(&line);
        let mut parser = Parser::from(&mut tokens);
        dbg!(parser.parse());
        dbg!(parser.errors());
    }
}
