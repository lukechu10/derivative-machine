//! Transforms the AST into its derivative.

use crate::parser::{BinOpKind, Expr, UnaryOpKind};

// Reading comments: k is a constant. u, v are variables
pub fn derivative(expr: &Expr, id: &str) -> Expr {
    match expr {
        // k' = 0
        Expr::Literal(_) => Expr::Literal(0.0),
        Expr::Identifier(ident) => {
            if ident == id {
                Expr::Literal(1.0)
            } else {
                Expr::Literal(0.0) // constant
            }
        }
        Expr::Binary { left, op, right } => match op {
            // (u + v)' = u' + v'
            BinOpKind::Plus => Expr::Binary {
                left: Box::new(derivative(left, id)),
                op: BinOpKind::Plus,
                right: Box::new(derivative(right, id)),
            },
            // (u - v)' = u' - v'
            BinOpKind::Minus => Expr::Binary {
                left: Box::new(derivative(left, id)),
                op: BinOpKind::Minus,
                right: Box::new(derivative(right, id)),
            },
            // (uv)' = u'v + uv'
            BinOpKind::Asterisk => Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(derivative(left, id)),
                    op: BinOpKind::Asterisk,
                    right: right.clone(),
                }),
                op: BinOpKind::Plus,
                right: Box::new(Expr::Binary {
                    left: left.clone(),
                    op: BinOpKind::Asterisk,
                    right: Box::new(derivative(right, id)),
                }),
            },
            // treat (u / v)' as (u * v ^ -1)'
            BinOpKind::Slash => derivative(
                &Expr::Binary {
                    left: left.clone(),
                    op: BinOpKind::Asterisk,
                    right: Box::new(Expr::Binary {
                        left: right.clone(),
                        op: BinOpKind::Exponent,
                        right: Box::new(Expr::Literal(-1.0)),
                    }),
                },
                id,
            ),
            // (u ^ k)' = ku ^ (k - 1)
            BinOpKind::Exponent => {
                if let box Expr::Literal(_) = right {
                    Expr::Binary {
                        left: right.clone(),
                        op: BinOpKind::Asterisk,
                        right: Box::new(Expr::Binary {
                            left: left.clone(),
                            op: BinOpKind::Exponent,
                            right: Box::new(Expr::Binary {
                                left: right.clone(),
                                op: BinOpKind::Minus,
                                right: Box::new(Expr::Literal(1.0)),
                            }),
                        }),
                    }
                } else {
                    todo!();
                }
            }
        },
        Expr::Unary { op, right } => match op {
            UnaryOpKind::Plus => derivative(&right, id),
            UnaryOpKind::Minus => Expr::Unary {
                op: UnaryOpKind::Minus,
                right: Box::new(derivative(&right, id)),
            },
        },
        Expr::Error => Expr::Error,
    }
}
