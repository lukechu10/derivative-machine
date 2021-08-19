use crate::lexer::Token;
use std::{convert::TryFrom, convert::TryInto, fmt, iter::Peekable};

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

impl fmt::Display for BinOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOpKind::Plus => "+",
                BinOpKind::Minus => "-",
                BinOpKind::Asterisk => "*",
                BinOpKind::Slash => "/",
                BinOpKind::Exponent => "^",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnaryOpKind {
    Minus,
}

impl TryFrom<Token> for UnaryOpKind {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Minus => Ok(UnaryOpKind::Minus),
            _ => Err(()),
        }
    }
}

impl fmt::Display for UnaryOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOpKind::Minus => "-",
            }
        )
    }
}

/// Represents an expression. To print out the expression in a human readable format, use the [`fmt::Display`] trait.

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
    Unary {
        op: UnaryOpKind,
        right: Box<Expr>,
    },
    // used when filling in invalid syntax
    Error,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(num) => {
                if *num >= 0.0 {
                    write!(f, "{}", num)
                } else {
                    // print negative number in paren
                    write!(f, "({})", num)
                }
            }
            Expr::Identifier(ident) => write!(f, "{}", ident),
            Expr::Binary { left, op, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Unary { op, right } => write!(f, "({}{})", op, right),
            Expr::Error => write!(f, "err"),
        }
    }
}

pub trait ExprVisitor: Sized {
    /// Callback when visiting an AST node.
    fn visit(&mut self, expr: &mut Expr) {
        walk_expr(expr, self);
    }
}

pub fn walk_expr(expr: &mut Expr, visitor: &mut impl ExprVisitor) {
    match expr {
        Expr::Literal(_) => {}
        Expr::Identifier(_) => {}
        Expr::Binary { left, op: _, right } => {
            visitor.visit(left.as_mut());
            visitor.visit(right.as_mut());
        }
        Expr::Unary { op: _, right } => {
            visitor.visit(right.as_mut());
        }
        Expr::Error => {}
    }
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
        let expr = self.parse_expr();
        if self.eat_tok() != Token::Eof {
            self.unexpected();
        }
        expr
    }

    /// Alias for `self.parse_expr_bp(0)` to accept any expression.
    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0)
    }

    fn parse_atom(&mut self) -> Expr {
        match self.eat_tok() {
            Token::Number(num) => Expr::Literal(num),
            Token::Identifier(ident) => Expr::Identifier(ident),
            Token::OpenParen => {
                let expr = self.parse_expr();
                match self.eat_tok() {
                    Token::CloseParen => expr,
                    _ => self.unexpected_expected("a '(' token"),
                }
            }
            _ => self.unexpected_expected("an expression"),
        }
    }

    fn parse_expr_bp(&mut self, min_bp: i32) -> Expr {
        let mut left = match self.current_tok.get_prefix_bp() {
            ((), -1) => self.parse_atom(), // not prefix
            ((), right_bp) => {
                let prefix_op: UnaryOpKind = self
                    .eat_tok()
                    .try_into()
                    .expect("non negative bp should be valid unary op");
                let right = self.parse_expr_bp(right_bp);
                if let Expr::Literal(num) = right {
                    // fold unary literal in ast
                    Expr::Literal(num * -1.0)
                } else {
                    Expr::Unary {
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
            left = Expr::Binary {
                left: Box::new(left),
                op: bin_op,
                right: Box::new(right),
            }
        }

        left
    }

    // utils

    /// Returns the current token. Sets `self.current_tok` to the next [`Token`] in the lexer.
    fn eat_tok(&mut self) -> Token {
        let res = self.current_tok.clone();
        self.current_tok = self.lexer.next().unwrap_or(Token::Eof);
        res
    }

    /// Returns [`Expr::Error`].
    fn unexpected(&mut self) -> Expr {
        self.errors.push("unexpected token".to_string());
        Expr::Error
    }

    /// Returns [`Expr::Error`].
    fn unexpected_expected(&mut self, expected: &str) -> Expr {
        self.errors
            .push(format!("unexpected token, expected {}", expected));
        Expr::Error
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use logos::Logos;

    use super::*;

    fn check(input: &str, expect: Expect) {
        let lexer = Token::lexer(input);
        let mut parser = Parser::from(lexer);
        let expr = parser.parse();

        let mut actual = expr.to_string();
        for error in parser.errors() {
            actual += &format!("\n[ERROR]: {}", error);
        }
        expect.assert_eq(&actual);
    }

    #[test]
    fn literal() {
        check("1", expect![[r#"1"#]]);
        check("-3", expect![[r#"(-3)"#]]);
    }

    #[test]
    fn vars() {
        check("x", expect![[r#"x"#]]);
        check("abc", expect![[r#"abc"#]]);
    }

    #[test]
    fn bin_ops() {
        check("1 + 2", expect![[r#"(1 + 2)"#]]);
        check("-3 - 4", expect![[r#"((-3) - 4)"#]]);
        check("1 + -2", expect![[r#"(1 + (-2))"#]]);
        check("-3 + -4", expect![[r#"((-3) + (-4))"#]]);
        check("-3 + -4", expect![[r#"((-3) + (-4))"#]]);

        check("1 + 2", expect![[r#"(1 + 2)"#]]);
        check("1 - 2", expect![[r#"(1 - 2)"#]]);
        check("1 * 2", expect![[r#"(1 * 2)"#]]);
        check("1 / 2", expect![[r#"(1 / 2)"#]]);
        check("1 ^ 2", expect![[r#"(1 ^ 2)"#]]);
        check("1 ** 2", expect![[r#"(1 ^ 2)"#]]);
    }

    #[test]
    fn paren() {
        check("(1)", expect![[r#"1"#]]);
        check("(-1)", expect![[r#"(-1)"#]]);

        check("(1 + 2)", expect![[r#"(1 + 2)"#]]);
        check("(1 + 2) * 3", expect![[r#"((1 + 2) * 3)"#]]);
        check("1 + (2 * 3)", expect![[r#"(1 + (2 * 3))"#]]);
    }

    #[test]
    fn precedence() {
        check("1 + 2 * 3", expect![[r#"(1 + (2 * 3))"#]]);
        check("1 + 2 - 3", expect![[r#"((1 + 2) - 3)"#]]);
        check("1 * 2 + 3 * 4", expect![[r#"((1 * 2) + (3 * 4))"#]]);
    }

    #[test]
    fn error_unknown_operator() {
        check(
            "1 $ 2",
            expect![[r#"
            1
            [ERROR]: unexpected token"#]],
        );
    }

    #[test]
    fn error_unmatched_paren() {
        check(
            "(1",
            expect![[r#"
                err
                [ERROR]: unexpected token, expected a '(' token"#]],
        );
        check(
            "(1 + 2",
            expect![[r#"
                err
                [ERROR]: unexpected token, expected a '(' token"#]],
        );
        check(
            "1)",
            expect![[r#"
            1
            [ERROR]: unexpected token"#]],
        );
        check(
            "1 + 2)",
            expect![[r#"
            (1 + 2)
            [ERROR]: unexpected token"#]],
        );
    }
}
