//! Lexer for rules in string format.

use crate::parser::{BinOpKind, UnaryOpKind};
use logos::Logos;
use std::convert::TryFrom;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum RuleToken {
    #[regex("[0-9.]+", |lex| lex.slice().parse())]
    Literal(f64),
    #[regex("_[0-9.]+", |lex| lex.slice()[1..].parse())]
    AnySubExpr(i32),
    #[regex("_lit[0-9.]+", |lex| lex.slice()[4..].parse())]
    AnyLiteral(i32),
    #[regex("_nonlit[0-9.]+", |lex| lex.slice()[7..].parse())]
    AnyNonLiteral(i32),
    // operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("**")]
    #[token("^")]
    Exponent,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl RuleToken {
    /// Returns the binding power for the binary (infix) operator or `(-1, -1)` if not a valid operator.
    pub fn get_infix_bp(&self) -> (i32, i32) {
        match self {
            RuleToken::Plus | RuleToken::Minus => (1, 2),
            RuleToken::Asterisk | RuleToken::Slash => (3, 4),
            RuleToken::Exponent => (6, 5), // right associative
            _ => (-1, -1),
        }
    }

    /// Returns the binding power for the prefix operator or `((), -1)` if not a valid operator.
    pub fn get_prefix_bp(&self) -> ((), i32) {
        match self {
            RuleToken::Plus | RuleToken::Minus => ((), 8),
            _ => ((), -1),
        }
    }
}

impl TryFrom<RuleToken> for BinOpKind {
    type Error = ();

    fn try_from(value: RuleToken) -> Result<Self, Self::Error> {
        match value {
            RuleToken::Plus => Ok(BinOpKind::Plus),
            RuleToken::Minus => Ok(BinOpKind::Minus),
            RuleToken::Asterisk => Ok(BinOpKind::Asterisk),
            RuleToken::Slash => Ok(BinOpKind::Slash),
            RuleToken::Exponent => Ok(BinOpKind::Exponent),
            _ => Err(()),
        }
    }
}

impl TryFrom<RuleToken> for UnaryOpKind {
    type Error = ();

    fn try_from(value: RuleToken) -> Result<Self, Self::Error> {
        match value {
            RuleToken::Plus => Ok(UnaryOpKind::Plus),
            RuleToken::Minus => Ok(UnaryOpKind::Minus),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuleToken::*;
    use super::*;

    #[test]
    fn test_lexer() {
        let tokens: Vec<_> = RuleToken::lexer("0 + _1").collect();
        assert_eq!(tokens, vec![Literal(0.0), Plus, AnySubExpr(1)]);

        let tokens: Vec<_> = RuleToken::lexer("_lit1 + _lit2").collect();
        assert_eq!(tokens, vec![AnyLiteral(1), Plus, AnyLiteral(2)]);
    }
}
