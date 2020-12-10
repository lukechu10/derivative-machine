//! Make expression more readable. For best result, pass expression through [`crate::transformations::Simplify`] before and after.

use crate::parser::{BinOpKind, Expr, UnaryOpKind};
use crate::{rule::MatchResult, transformations::RuleTransformSet};

#[must_use]
pub fn derivative(expr: &Expr) -> Expr {
    let transforms = RuleTransformSet::new_from_str(
        &[("_lit1", "0")],
        &[
            (
                "_1",
                &|res: &MatchResult| match res.matched_exprs.get(&1).unwrap() {
                        Expr::Identifier(id) if id == "x" /* TODO */ => Some(Expr::Literal(1.0)),
                        _ => None,
                    },
            ),
            // unary minus
            ("-_1", &|res: &MatchResult| {
                Some(Expr::Unary {
                    op: UnaryOpKind::Minus,
                    right: Box::new(derivative(res.matched_exprs.get(&1).unwrap())),
                })
            }),
            ("_1 + _2", &|res: &MatchResult| {
                Some(Expr::Binary {
                    left: Box::new(derivative(res.matched_exprs.get(&1).unwrap())),
                    op: BinOpKind::Plus,
                    right: Box::new(derivative(res.matched_exprs.get(&2).unwrap())),
                })
            }),
            ("_1 * _2", &|res: &MatchResult| {
                Some(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(derivative(res.matched_exprs.get(&1).unwrap())),
                        op: BinOpKind::Asterisk,
                        right: Box::new((*res.matched_exprs.get(&2).unwrap()).clone()),
                    }),
                    op: BinOpKind::Plus,
                    right: Box::new(Expr::Binary {
                        left: Box::new(derivative(res.matched_exprs.get(&2).unwrap())),
                        op: BinOpKind::Asterisk,
                        right: Box::new((*res.matched_exprs.get(&1).unwrap()).clone()),
                    }),
                })
            }),
            ("_1 / _2", &|res: &MatchResult| {
                Some(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(derivative(res.matched_exprs.get(&1).unwrap())),
                            op: BinOpKind::Asterisk,
                            right: Box::new((*res.matched_exprs.get(&2).unwrap()).clone()),
                        }),
                        op: BinOpKind::Minus,
                        right: Box::new(Expr::Binary {
                            left: Box::new(derivative(res.matched_exprs.get(&2).unwrap())),
                            op: BinOpKind::Asterisk,
                            right: Box::new((*res.matched_exprs.get(&1).unwrap()).clone()),
                        }),
                    }),
                    op: BinOpKind::Slash,
                    right: Box::new(Expr::Binary {
                        left: Box::new((*res.matched_exprs.get(&2).unwrap()).clone()),
                        op: BinOpKind::Exponent,
                        right: Box::new(Expr::Literal(2.0)),
                    }),
                })
            }),
            // use chain rule g(x) ^ n => n * g(x) ^ (n - 1) * g'(x)
            ("_1 ^ _2", &|res: &MatchResult| {
                Some(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new((*res.matched_exprs.get(&2).unwrap()).clone()),
                        op: BinOpKind::Asterisk,
                        right: Box::new(Expr::Binary {
                            left: Box::new((*res.matched_exprs.get(&1).unwrap()).clone()),
                            op: BinOpKind::Exponent,
                            right: Box::new(Expr::Binary {
                                left: Box::new((*res.matched_exprs.get(&2).unwrap()).clone()),
                                op: BinOpKind::Minus,
                                right: Box::new(Expr::Literal(1.0)),
                            }),
                        }),
                    }),
                    op: BinOpKind::Asterisk,
                    right: Box::new(derivative(res.matched_exprs.get(&1).unwrap())),
                })
            }),
            // catch all
            ("_1", &|_res| Some(Expr::Error)),
        ],
    );

    transforms
        .apply_rules_once(expr)
        .expect(&format!("derivative not yet implemented for {}", expr))
}
