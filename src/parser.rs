use crate::lexer::Token;
use std::{borrow::Cow, convert::TryFrom, iter::Peekable};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinOpKind {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Exponent,
}

impl<'a> TryFrom<Token<'a>> for BinOpKind {
    type Error = ();

    fn try_from(value: Token<'a>) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(BinOpKind::Plus),
            Token::Minus => Ok(BinOpKind::Minus),
            Token::Asterisk => Ok(BinOpKind::Asterisk),
            Token::Slash => Ok(BinOpKind::Slash),
            Token::Exponent => Ok(BinOpKind::Exponent),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    // atoms
    Literal(f64),
    Identifier(Cow<'a, str>),
    // complexe
    Binary {
        left: Box<Expr<'a>>,
        op: BinOpKind,
        right: Box<Expr<'a>>,
    },
    // used when filling in invalid syntax
    Error,
}

pub struct Parser<'a, T>
where
    T: Iterator<Item = Token<'a>>,
{
    lexer: Peekable<T>,
    current_tok: Token<'a>,
    errors: Vec<String>,
}

impl<'a, T> From<T> for Parser<'a, T>
where
    T: Iterator<Item = Token<'a>>,
{
    fn from(lexer: T) -> Self {
        let mut lexer = lexer.peekable();
        let current_tok = lexer
            .next()
            .expect("there should be at least 1 element in lexer");
        Self {
            lexer,
            current_tok,
            errors: Vec::new(),
        }
    }
}

impl<'a, T> Parser<'a, T>
where
    T: Iterator<Item = Token<'a>>,
{
    pub fn parse(&mut self) -> Expr {
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Expr {
        match self.eat_tok() {
            Token::Number(num) => Expr::Literal(num),
            Token::Identifier(ident) => Expr::Identifier(ident.into()),
            _ => self.unexpected("an expression"),
        }
    }

    // fn parse_expr_bp(&mut self, min_bp: i32) -> Expr {
    //     let mut lhs = self.parse_atom();

    //     loop {
    //         let (left_bp, right_bp) = self.current_tok.get_bp();

    //         // stop parsing
    //         if left_bp < min_bp {
    //             break;
    //         }
    //     }

    //     lhs
    // }

    // utils

    /// Returns the current token. Sets `self.current_tok` to the next [`Token`] in the lexer.
    fn eat_tok(&mut self) -> Token<'a> {
        let res = self.current_tok;
        self.current_tok = self.lexer.next().unwrap_or(Token::Error);
        res
    }

    /// Returns [`Expr::Error`].
    fn unexpected(&mut self, expected: &str) -> Expr {
        self.errors
            .push(format!("unexpected token, expected {}", expected));
        Expr::Error
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }
}
