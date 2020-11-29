//! Fold constants in AST.

use crate::parser::{walk_expr, BinOpKind, Expr, ExprVisitor, UnaryOpKind};

/// Runs one fold pass on the AST.
/// The resulting AST may still be further folded. Keep running the pass until `last_pass_folded` remains false.
/// Prefer using [`FoldVisitor`] instead.
pub struct FoldVisitor;

impl ExprVisitor for FoldVisitor {
    fn visit(&mut self, expr: &mut Expr) {
        walk_expr(expr, self);

        match expr {
            // fold multiplication / division with 0
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Asterisk | BinOpKind::Slash,
                right: _,
            } if *left == 0.0 => {
                *expr = Expr::Literal(0.0);
            }
            Expr::Binary {
                left: _,
                op: BinOpKind::Asterisk,
                right: box Expr::Literal(right),
            } if *right == 0.0 => {
                *expr = Expr::Literal(0.0);
            }
            // fold addition with 0
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Plus | BinOpKind::Minus,
                right,
            } if *left == 0.0 => {
                *expr = *right.clone();
            }
            Expr::Binary {
                left,
                op: BinOpKind::Plus | BinOpKind::Minus,
                right: box Expr::Literal(right),
            } if *right == 0.0 => {
                *expr = *left.clone();
            }
            // fold multiplication / division with 1
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Asterisk,
                right,
            } if *left == 1.0 => {
                *expr = *right.clone();
            }
            Expr::Binary {
                left,
                op: BinOpKind::Asterisk | BinOpKind::Slash,
                right: box Expr::Literal(right),
            } if *right == 1.0 => {
                *expr = *left.clone();
            }
            // fold exponentiation with 1
            Expr::Binary {
                left,
                op: BinOpKind::Exponent,
                right: box Expr::Literal(right),
            } if *right == 1.0 => {
                *expr = *left.clone();
            }
            // fold double exponent, e.g. (x ^ 2) ^ 3 = x ^ 6
            Expr::Binary {
                left:
                    box Expr::Binary {
                        left,
                        op: BinOpKind::Exponent,
                        right: box Expr::Literal(inner),
                    },
                op: BinOpKind::Exponent,
                right: box Expr::Literal(outer),
            } => {
                *expr = Expr::Binary {
                    left: left.clone(),
                    op: BinOpKind::Exponent,
                    right: Box::new(Expr::Literal(*inner * *outer)),
                }
            }
            // fold binop with two constants
            Expr::Binary {
                left: box Expr::Literal(left_lit),
                op,
                right: box Expr::Literal(right_lit),
            } => {
                *expr = Expr::Literal(match op {
                    BinOpKind::Plus => *left_lit + *right_lit,
                    BinOpKind::Minus => *left_lit - *right_lit,
                    BinOpKind::Asterisk => *left_lit * *right_lit,
                    BinOpKind::Slash => *left_lit / *right_lit,
                    BinOpKind::Exponent => left_lit.powf(*right_lit),
                });
            }
            // fold unary op on Literal into signed Literal
            Expr::Unary {
                op,
                right: box Expr::Literal(right_lit),
            } => {
                *expr = Expr::Literal(match op {
                    UnaryOpKind::Plus => *right_lit,
                    UnaryOpKind::Minus => -*right_lit,
                });
            }
            // fold same identifier add into multiplication, e.g. x + x = 2x
            // TODO: fold left and right with same power, e.g. (x ^ 2) + (3x ^ 2)
            Expr::Binary {
                left: box Expr::Identifier(left_id),
                op: BinOpKind::Plus,
                right: box Expr::Identifier(right_id),
            } => {
                if left_id == right_id {
                    *expr = Expr::Binary {
                        left: Box::new(Expr::Literal(2.0)),
                        op: BinOpKind::Asterisk,
                        right: Box::new(Expr::Identifier(left_id.clone())),
                    };
                }
            }
            _ => {}
        }
    }
}
