//! AST transformations.

pub mod simplify;

use crate::parser::Expr;
use crate::rule::parser::RuleExpr;
use crate::rule::MatchResult;

pub enum TransformOut<'a> {
    OutPattern(RuleExpr),
    OutHandler(&'a (dyn Fn(&MatchResult) -> Expr + Sync)),
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
            &'a (dyn for<'r, 's> Fn(&'r MatchResult<'s>) -> Expr + Sync + 'a),
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
                        TransformOut::OutHandler(handler) => expr = handler(&match_res),
                    }
                }
            }

            if !last_iter_transformed {
                break expr;
            }
        }
    }
}
