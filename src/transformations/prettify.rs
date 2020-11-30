//! Make expression more readable. For best result, pass expression through [`crate::transformations::Simplify`] before and after.

use crate::parser::{walk_expr, Expr, ExprVisitor};
use crate::transformations::RuleTransformSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref PRETTIFY_TRANSFORMS: RuleTransformSet<'static> = RuleTransformSet::new_from_str(
        &[("_1 ^ -_lit2", "1 / _1 ^ _lit2"), ("0.5", "1 / 2"),],
        &[]
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
