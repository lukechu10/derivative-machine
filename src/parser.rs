use crate::lexer::Token;
use std::{convert::TryFrom, convert::TryInto, iter::Peekable};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinOpKind {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Exponent,
}

impl TryFrom<Token> for BinOpKind {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
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
pub enum Expr {
    // atoms
    Literal(f64),
    Identifier(String),
    // complex
    Binary {
        left: Box<Expr>,
        op: BinOpKind,
        right: Box<Expr>,
    },
    // used when filling in invalid syntax
    Error,
}

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    lexer: Peekable<T>,
    current_tok: Token,
    errors: Vec<String>,
}

impl<T> From<T> for Parser<T>
where
    T: Iterator<Item = Token>,
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

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn parse(&mut self) -> Expr {
        self.parse_expr()
    }

    /// Alias for `self.parse_expr_bp(0)` to accept any expression.
    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    fn parse_atom(&mut self) -> Expr {
        match self.eat_tok() {
            Token::Number(num) => Expr::Literal(num),
            Token::Identifier(ident) => Expr::Identifier(ident.into()),
            _ => self.unexpected("an expression"),
        }
    }

    fn parse_expr_bp(&mut self, min_bp: i32) -> Expr {
        let mut left = self.parse_atom();

        loop {
            let (left_bp, right_bp) = self.current_tok.get_bp();

            // stop parsing
            if left_bp < min_bp {
                break;
            }
            let binop: BinOpKind = self
                .eat_tok()
                .try_into()
                .expect("non negative bp should be valid binop");
            let right = self.parse_expr_bp(right_bp);
            left = Expr::Binary {
                left: Box::new(left),
                op: binop,
                right: Box::new(right),
            }
        }

        left
    }

    // utils

    /// Returns the current token. Sets `self.current_tok` to the next [`Token`] in the lexer.
    fn eat_tok(&mut self) -> Token {
        let res = self.current_tok.clone();
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
