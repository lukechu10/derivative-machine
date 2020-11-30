//! Logic for matching a pattern and replacing with new `Expr`.

pub mod lexer;
pub mod parser;

use crate::parser::{Expr, UnaryOpKind};
use logos::Logos;
use parser::RuleExpr;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct MatchResult<'a> {
    /// `true` if the match was successful, `false` otherwise.
    pub matches: bool,
    /// Original expression (input).
    pub source_expr: &'a Expr,
    /// A list of matched wildcards.
    /// A failed match does not necessarily mean `matched_exprs` is empty. For instance, if a wildcard is successfully matched, then a fail occurs, the wildcard result will still be kept.
    /// All wildcard ids should be unique.
    pub matched_exprs: BTreeMap<i32, &'a Expr>,
}

impl RuleExpr {
    pub fn new_rule_from_str(pattern: &str) -> Self {
        let tokens = lexer::RuleToken::lexer(pattern);
        let mut parser = parser::RuleParser::from(tokens);
        parser.parse()
    }

    /// Tries to match a [`RuleExpr`] pattern on an [`Expr`].
    /// When encountering a wildcard rule, will append the matched [`Expr`] onto the `matched_exprs` argument.
    /// # Params
    /// * `expr` - The expression to try to match.
    /// * `matched_exprs` - A list of matched expressions from wildcards.
    fn match_expr_inner<'a>(
        &self,
        expr: &'a Expr,
        matched_exprs: &mut BTreeMap<i32, &'a Expr>,
    ) -> bool {
        // Returns true if wildcard is successful match (no match with same id yet, or already matched same Expr).
        // Else returns false.
        let mut insert_added_match = |id: i32, expr: &'a Expr| {
            let existing = matched_exprs.get(&id);
            if existing.is_none() || existing.unwrap() == &expr {
                matched_exprs.insert(id, expr);
                true
            } else {
                false
            }
        };

        match self {
            RuleExpr::Literal(num_rule) => matches!(expr, Expr::Literal(num) if num == num_rule),
            RuleExpr::AnySubExpr(id) => insert_added_match(*id, expr),
            RuleExpr::AnyLiteral(id) => match expr {
                Expr::Literal(_) => insert_added_match(*id, expr),
                _ => false,
            },
            RuleExpr::AnyNonLiteral(id) => match expr {
                Expr::Literal(_) => false,
                _ => insert_added_match(*id, expr),
            },
            RuleExpr::Binary {
                left: left_rule,
                op: op_rule,
                right: right_rule,
            } => matches!(
                expr,
                Expr::Binary {
                    left,
                    op,
                    right
                } if op == op_rule && left_rule.match_expr_inner(left, matched_exprs) && right_rule.match_expr_inner(right, matched_exprs)
            ),
            RuleExpr::Unary {
                op: op_rule,
                right: right_rule,
            } => match expr {
                _ => {
                    matches!(expr, Expr::Unary {op, right} if op == op_rule && right_rule.match_expr_inner(right, matched_exprs))
                }
            },
            RuleExpr::Error => false,
        }
    }

    /// Tries to match a [`RuleExpr`] pattern on an [`Expr`].
    /// # Panics
    /// This method panics if two wildcard matches have the same id.
    pub fn match_expr<'a>(&self, expr: &'a Expr) -> MatchResult<'a> {
        let mut matched_exprs = BTreeMap::new();
        let matches = self.match_expr_inner(expr, &mut matched_exprs);
        MatchResult {
            matches,
            source_expr: expr,
            matched_exprs,
        }
    }

    /// Fills in the wildcards of a [`RuleExpr`] with results of `match_res`.
    /// # Panics
    /// This method panics if a wildcard id is not found in `matched_exprs`. This method also panics if the wildcard type does not match.
    pub fn write_expr(&self, matched_exprs: &BTreeMap<i32, &Expr>) -> Expr {
        match self {
            RuleExpr::Literal(num) => Expr::Literal(*num),
            RuleExpr::AnySubExpr(id) => (*matched_exprs
                .get(id)
                .expect(&format!("wildcard _{} not found", id)))
            .clone(),
            RuleExpr::AnyLiteral(id) => (*matched_exprs
                .get(id)
                .expect(&format!("wildcard _lit{} not found", id)))
            .clone(),
            RuleExpr::AnyNonLiteral(id) => (*matched_exprs
                .get(id)
                .expect(&format!("wildcard _nonlit{} not found", id)))
            .clone(),
            RuleExpr::Binary {
                left: left_rule,
                op,
                right: right_rule,
            } => Expr::Binary {
                left: Box::new(left_rule.write_expr(matched_exprs)),
                op: *op,
                right: Box::new(right_rule.write_expr(matched_exprs)),
            },
            RuleExpr::Unary {
                op,
                right: right_rule,
            } => {
                if *op == UnaryOpKind::Minus {
                    match **right_rule {
                        // if literal or literal wildcard, fold directly in emitted ast
                        RuleExpr::Literal(num) => Expr::Literal(-num),
                        RuleExpr::AnyLiteral(id) => match *matched_exprs
                            .get(&id)
                            .expect(&format!("wildcard _lit{} not found", id))
                        {
                            Expr::Literal(num) => Expr::Literal(-num),
                            _ => unreachable!(),
                        },
                        _ => Expr::Unary {
                            op: *op,
                            right: Box::new(right_rule.write_expr(matched_exprs)),
                        },
                    }
                } else {
                    // emit right ast as is
                    right_rule.write_expr(matched_exprs)
                }
            }
            RuleExpr::Error => Expr::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parser::Parser;

    fn expr_matches_rule(expr: &str, rule: &str) -> bool {
        let expr: Expr = Parser::from(Token::lexer(expr)).parse();
        let rule = RuleExpr::new_rule_from_str(rule);
        rule.match_expr(&expr).matches
    }

    fn rule_transform_expr(expr: &str, rule: &str, out: &str, expected: &str) {
        let expr: Expr = Parser::from(Token::lexer(expr)).parse();
        let rule = RuleExpr::new_rule_from_str(rule);
        let out = RuleExpr::new_rule_from_str(out);
        let expected: Expr = Parser::from(Token::lexer(expected)).parse();

        let match_res = rule.match_expr(&expr);
        assert!(match_res.matches);

        let out_expr = out.write_expr(&match_res.matched_exprs);
        assert_eq!(out_expr, expected);
    }

    #[test]
    fn test_match_expr_any_sub_expr() {
        assert!(expr_matches_rule("0 + x", "0 + _1"));
        assert!(expr_matches_rule("0 + 2 * x", "0 + _1"));
        assert!(!expr_matches_rule("1 + x", "0 + _1"));
        assert!(!expr_matches_rule("(0 + 2) * x", "0 + _1"));
    }

    #[test]
    fn test_match_expr_any_literal() {
        assert!(expr_matches_rule("2 + x", "_lit1 + _2"));
        assert!(!expr_matches_rule("x + x", "_lit1 + _2"));
        assert!(!expr_matches_rule("(2 * x) + x", "_lit1 + _2"));
    }

    #[test]
    fn test_write_expr() {
        rule_transform_expr("0 + x", "0 + _1", "_1", "x");
        rule_transform_expr("x * 20", "_1 * _lit2", "_lit2 * _1", "20 * x");
        rule_transform_expr(
            "(x ^ 2) ^ 3",
            "(_1 ^ _lit2) ^ _lit3",
            "_1 ^ (_lit2 * _lit3)",
            "x ^ (2 * 3)",
        );
        rule_transform_expr("1 / x", "1 / _1", "_1 ^ -1", "x ^ -1");
    }
}
