//! Make expression more readable. For best result, pass expression through [`crate::transformations::Simplify`] before and after.

use crate::transformations::RuleTransformSet;
use crate::{
    parser::{walk_expr, Expr, ExprVisitor},
    rule::parser::RuleExpr,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref PRETTIFY_TRANSFORMS: RuleTransformSet<'static> = RuleTransformSet::new_from_str(
        &[("0.5", "1 / 2"),],
        &[
            // change negative exponent to division
            ("_1 ^ _lit2",
            &|res| match res.matched_exprs.get(&2).unwrap() {
                Expr::Literal(num) =>
                    if *num < 0.0 {
                        Some(
                            RuleExpr::new_rule_from_str("1 / _1 ^ -_lit2")
                                .write_expr(&res.matched_exprs),
                        )
                    } else {
                        None
                    },
                _ => unreachable!(),
            })
        ]
    );
}

pub struct Prettify;

impl ExprVisitor for Prettify {
    fn visit(&mut self, expr: &mut Expr) {
        walk_expr(expr, self);

        *expr = PRETTIFY_TRANSFORMS.apply_rules(expr);

        // simplify any newly created ast nodes
        walk_expr(expr, self);
    }
}
