//! AST transformations.

pub mod prettify;
pub mod simplify;

use crate::parser::Expr;
use crate::rule::parser::RuleExpr;
use crate::rule::MatchResult;

/// The max number of iterations per apply. Exceeding this amount will cause an error.
pub const MAX_ITERATIONS_PER_APPLY: i32 = 500;

pub enum TransformOut<'a> {
    OutPattern(RuleExpr),
    /// A handler to transform the [`Expr`]. If the handler returns `None`, the expression is not modified.
    OutHandler(&'a (dyn Fn(&MatchResult) -> Option<Expr> + Sync)),
}

pub struct Transformation<'a> {
    pattern: RuleExpr,
    out: TransformOut<'a>,
}

/// Utility to keep on applying transformations until no more matches.
pub struct RuleTransformSet<'a> {
    pub rules: Vec<Transformation<'a>>,
}

impl<'a> RuleTransformSet<'a> {
    pub fn new_from_str(
        patterns: &[(&str, &str)],
        handlers: &'a [(
            &'a str,
            &'a (dyn for<'r, 's> Fn(&'r MatchResult<'s>) -> Option<Expr> + Sync + 'a),
        )],
    ) -> Self {
        let mut transformations: Vec<_> = patterns
            .iter()
            .map(|(pattern, out)| {
                let pattern = RuleExpr::new_rule_from_str(pattern);
                let out = RuleExpr::new_rule_from_str(out);
                Transformation {
                    pattern,
                    out: TransformOut::OutPattern(out),
                }
            })
            .collect();

        transformations.extend(handlers.iter().map(|(pattern, handler)| {
            let pattern = RuleExpr::new_rule_from_str(pattern);
            Transformation {
                pattern,
                out: TransformOut::OutHandler(handler),
            }
        }));

        Self {
            rules: transformations,
        }
    }

    pub fn apply_rules(&self, expr: &Expr) -> Expr {
        let mut expr = expr.clone();
        let mut i = 0;
        loop {
            let mut last_iter_transformed = false;

            for transform in &self.rules {
                // match pattern
                let match_res = transform.pattern.match_expr(&expr);
                if match_res.matches {
                    last_iter_transformed = true;

                    // write output
                    match &transform.out {
                        TransformOut::OutPattern(out) => {
                            expr = out.write_expr(&match_res.matched_exprs)
                        }
                        TransformOut::OutHandler(handler) => match handler(&match_res) {
                            Some(res) => expr = res,
                            None => last_iter_transformed = false, // if handler returned `None`, no change happened
                        },
                    }
                }
            }

            if !last_iter_transformed {
                break expr;
            } else if i > MAX_ITERATIONS_PER_APPLY {
                log::warn!("Exceeded MAX_ITERATIONS_PER_APPLY, exiting immediately");
                break expr;
            }

            i += 1;
        }
    }
}
