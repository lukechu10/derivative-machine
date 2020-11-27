//! Fold constants in AST.

use crate::parser::{walk_expr, BinOpKind, Expr, ExprVisitor, UnaryOpKind};

/// Fold literals (`1 + 1` becomes `2`).
pub struct FoldVisitor;

impl ExprVisitor for FoldVisitor {
    fn visit(&mut self, expr: &mut Expr) {
        let mut visitor_internal = FoldVisitorInternal {
            last_pass_folded: true,
        };

        while visitor_internal.last_pass_folded {
            visitor_internal.last_pass_folded = false;
            visitor_internal.visit(expr);
        }
    }
}

/// Runs one fold pass on the AST.
/// The resulting AST may still be further folded. Keep running the pass until `last_pass_folded` remains false.
/// Prefer using [`FoldVisitor`] instead.
struct FoldVisitorInternal {
    /// Whether the last pass folded any constants. The top level visit function should keep on running the pass until it is false.
    last_pass_folded: bool,
}

impl ExprVisitor for FoldVisitorInternal {
    fn visit(&mut self, expr: &mut Expr) {
        match expr {
            // fold multiplication / division with 0
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Asterisk | BinOpKind::Slash,
                right: _,
            } if *left == 0.0 => {
                self.last_pass_folded = true;
                *expr = Expr::Literal(0.0);
            }
            Expr::Binary {
                left: _,
                op: BinOpKind::Asterisk,
                right: box Expr::Literal(right),
            } if *right == 0.0 => {
                self.last_pass_folded = true;
                *expr = Expr::Literal(0.0);
            }
            // fold addition with 0
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Plus | BinOpKind::Minus,
                right,
            } if *left == 0.0 => {
                self.last_pass_folded = true;
                *expr = *right.clone();
            }
            Expr::Binary {
                left,
                op: BinOpKind::Plus | BinOpKind::Minus,
                right: box Expr::Literal(right),
            } if *right == 0.0 => {
                self.last_pass_folded = true;
                *expr = *left.clone();
            }
            // fold multiplication / division with 1
            Expr::Binary {
                left: box Expr::Literal(left),
                op: BinOpKind::Asterisk,
                right,
            } if *left == 1.0 => {
                self.last_pass_folded = true;
                *expr = *right.clone();
            }
            Expr::Binary {
                left,
                op: BinOpKind::Asterisk | BinOpKind::Slash,
                right: box Expr::Literal(right),
            } if *right == 1.0 => {
                self.last_pass_folded = true;
                *expr = *left.clone();
            }
            // fold exponentiation with 1
            Expr::Binary {
                left,
                op: BinOpKind::Exponent,
                right: box Expr::Literal(right),
            } if *right == 1.0 => {
                self.last_pass_folded = true;
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
                self.last_pass_folded = true;
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
                self.last_pass_folded = true;
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
                self.last_pass_folded = true;
                *expr = Expr::Literal(match op {
                    UnaryOpKind::Plus => *right_lit,
                    UnaryOpKind::Minus => -*right_lit,
                });
            }
            _ => {}
        }
        walk_expr(expr, self);
    }
}
