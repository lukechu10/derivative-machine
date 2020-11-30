//! Parsing for rules in string format.

use crate::parser::{BinOpKind, UnaryOpKind};
use crate::rule::lexer::RuleToken;
use std::{convert::TryInto, fmt, iter::Peekable};

/// Represents an rule expression. To print out the rule expression in a human readable format, use the `Display::fmt` trait.
/// # Example
///
/// ```
/// let rule = RuleExpr::Literal(3);
/// assert_eq!(std::fmt::Display::fmt(rule).unwrap(), "3");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum RuleExpr {
    // atoms
    Literal(f64),
    AnySubExpr(i32),
    AnyLiteral(i32),
    AnyNonLiteral(i32),
    // complex
    Binary {
        left: Box<RuleExpr>,
        op: BinOpKind,
        right: Box<RuleExpr>,
    },
    Unary {
        op: UnaryOpKind,
        right: Box<RuleExpr>,
    },
    // used when filling in invalid syntax
    Error,
}

impl fmt::Display for RuleExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleExpr::Literal(num) => {
                if *num >= 0.0 {
                    write!(f, "{}", num)
                } else {
                    // print negative number in paren
                    write!(f, "({})", num)
                }
            }
            RuleExpr::AnySubExpr(id) => write!(f, "_{}", id),
            RuleExpr::AnyLiteral(id) => write!(f, "_lit{}", id),
            RuleExpr::AnyNonLiteral(id) => write!(f, "_nonlit{}", id),
            RuleExpr::Binary { left, op, right } => write!(f, "({} {} {})", left, op, right),
            RuleExpr::Unary { op, right } => write!(f, "({}{})", op, right),
            RuleExpr::Error => write!(f, "err"),
        }
    }
}

pub struct RuleParser<T>
where
    T: Iterator<Item = RuleToken>,
{
    lexer: Peekable<T>,
    current_tok: RuleToken,
    errors: Vec<String>,
}

impl<T> From<T> for RuleParser<T>
where
    T: Iterator<Item = RuleToken>,
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

impl<T> RuleParser<T>
where
    T: Iterator<Item = RuleToken>,
{
    pub fn parse(&mut self) -> RuleExpr {
        self.parse_expr()
    }

    /// Alias for `self.parse_expr_bp(0)` to accept any expression.
    fn parse_expr(&mut self) -> RuleExpr {
        self.parse_expr_bp(0)
    }

    fn parse_atom(&mut self) -> RuleExpr {
        match self.eat_tok() {
            RuleToken::Literal(num) => RuleExpr::Literal(num),
            RuleToken::AnySubExpr(id) => RuleExpr::AnySubExpr(id),
            RuleToken::AnyLiteral(id) => RuleExpr::AnyLiteral(id),
            RuleToken::AnyNonLiteral(id) => RuleExpr::AnyNonLiteral(id),
            RuleToken::OpenParen => {
                let expr = self.parse_expr();
                match self.eat_tok() {
                    RuleToken::CloseParen => expr,
                    _ => self.unexpected("a '(' token"),
                }
            }
            _ => self.unexpected("a rule expression"),
        }
    }

    fn parse_expr_bp(&mut self, min_bp: i32) -> RuleExpr {
        let mut left = match self.current_tok.get_prefix_bp() {
            ((), -1) => self.parse_atom(), // not prefix
            ((), right_bp) => {
                let prefix_op: UnaryOpKind = self
                    .eat_tok()
                    .try_into()
                    .expect("non negative bp should be valid unary op");
                let right = self.parse_expr_bp(right_bp);
                if let RuleExpr::Literal(num) = right {
                    // fold unary literal in ast
                    RuleExpr::Literal(num * -1.0)
                } else {
                    RuleExpr::Unary {
                        op: prefix_op,
                        right: Box::new(right),
                    }
                }
            }
        };

        loop {
            let (left_bp, right_bp) = self.current_tok.get_infix_bp();

            // stop parsing
            if left_bp < min_bp {
                break;
            }
            let bin_op: BinOpKind = self
                .eat_tok()
                .try_into()
                .expect("non negative bp should be valid binop");
            let right = self.parse_expr_bp(right_bp);
            left = RuleExpr::Binary {
                left: Box::new(left),
                op: bin_op,
                right: Box::new(right),
            }
        }

        left
    }

    // utils

    /// Returns the current token. Sets `self.current_tok` to the next [`RuleToken`] in the lexer.
    fn eat_tok(&mut self) -> RuleToken {
        let res = self.current_tok.clone();
        self.current_tok = self.lexer.next().unwrap_or(RuleToken::Error);
        res
    }

    /// Returns [`crate::parser::Expr::Error`].
    fn unexpected(&mut self, expected: &str) -> RuleExpr {
        self.errors
            .push(format!("unexpected token, expected {}", expected));
        RuleExpr::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_parser() {
        let tokens = RuleToken::lexer("0 + _1");
        let mut parser = RuleParser::from(tokens);
        assert_eq!(
            parser.parse(),
            RuleExpr::Binary {
                left: Box::new(RuleExpr::Literal(0.0)),
                op: BinOpKind::Plus,
                right: Box::new(RuleExpr::AnySubExpr(1))
            }
        );

        let tokens = RuleToken::lexer("_lit1 + _lit2");
        let mut parser = RuleParser::from(tokens);
        assert_eq!(
            parser.parse(),
            RuleExpr::Binary {
                left: Box::new(RuleExpr::AnyLiteral(1)),
                op: BinOpKind::Plus,
                right: Box::new(RuleExpr::AnyLiteral(2))
            }
        );

        let tokens = RuleToken::lexer("(_lit1 + _lit2)");
        let mut parser = RuleParser::from(tokens);
        assert_eq!(
            parser.parse(),
            RuleExpr::Binary {
                left: Box::new(RuleExpr::AnyLiteral(1)),
                op: BinOpKind::Plus,
                right: Box::new(RuleExpr::AnyLiteral(2))
            }
        );
    }
}
