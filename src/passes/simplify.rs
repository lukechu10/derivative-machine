//! Fold constants in AST

use crate::parser::{walk_expr, BinOpKind, Expr, ExprVisitor};

/// Fold literals (`1 + 1` becomes `2`).
pub struct SimplifyVisitor;

impl ExprVisitor for SimplifyVisitor {
    fn visit(&mut self, expr: &mut Expr) {
        let mut visitor_internal = SimplifyVisitorInternal {
            last_pass_folded: true,
        };

        while visitor_internal.last_pass_folded {
            visitor_internal.last_pass_folded = false;
            visitor_internal.visit(expr);
        }
    }
}

/// Runs one simplify pass on the AST.
/// The resulting AST may still be further simplified and folded. Keep running the pass until `last_pass_folded` remains false.
struct SimplifyVisitorInternal {
    /// Whether the last pass folded any constants. The top level visit function should keep on running the pass until it is false.
    last_pass_folded: bool,
}

impl ExprVisitor for SimplifyVisitorInternal {
    fn visit(&mut self, expr: &mut Expr) {
        match expr {
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
            _ => {}
        }

        walk_expr(expr, self);
    }
}
