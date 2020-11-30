//! Fold constants.

use crate::parser::{walk_expr, Expr, ExprVisitor};
use crate::transformations::RuleTransformSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref SIMPLIFY_TRANSFORMS: RuleTransformSet<'static> = RuleTransformSet::new_from_str(&[
        // addition with 0
        ("0 + _1", "_1"),
        ("_1 + 0", "_1"),
        // multiplication with 0
        ("0 * _1", "0"),
        ("_1 * 0", "0"),
        // multiplication with 1
        ("1 * _1", "_1"),
        ("_1 * 1", "_1"),
        // division with 1
        ("_1 / 1", "_1"),

        ("_1 - _1", "0"),
        ("_1 + -_1", "0"),
        ("_1 / _1", "1"),
        ("_1 + _1", "2 * _1"),

        // exponentiation identities
        ("_1 ^ 0", "1"),
        ("_1 ^ 1", "_1"),
        ("1 ^ _1", "1"),
        // ("_1 ^ -1", "1 / _1"),
        ("(_1 ^ _lit2) ^ _lit3", "_1 ^ (_lit2 * _lit3)"), // fold double exponent, e.g. (x ^ 2) ^ 3 = x ^ 6
        ("(_1 ^ _2) * (_1 ^ _3)", "_1 ^ (_2 + _3)"),

        ("(_lit1 * _2) / _lit1", "_2"),
        ("(_lit1 * _2) / _lit3", "(_lit1 / _lit3) * _2"),

        ("(_2 * _1) + _1", "_1 * (_2 + 1)"),

        // simplify operations with commutativity, e.g. 2 * (3 * x) => 6 * x
        ("_lit1 + (_lit2 + _3)", "(_lit1 + _lit2) + _3"), // addition
        ("_lit1 * (_lit2 * _3)", "(_lit1 * _lit2) * _3"), // multiplication
        ("_lit1 * (_lit2 / _3)", "(_lit1 * _lit2) / _3"), // multiplication

        // for normalization purposes
        // ("(_1 + _2) + _3", "_1 + (_2 + _3)"),
        // ("(_1 * _2) * _3", "_1 * (_2 * _3)"),

        // move literals to left and rest to right, e.g. x * 2 => 2 * x
        ("_nonlit1 + _lit2", "_lit2 + _nonlit1"),
        ("_1 - _lit2", "-_lit2 + _1"), // change minus into plus to fold in one step
        ("_nonlit1 * _lit2", "_lit2 * _nonlit1"),
    ], &[
        // fold aritmatic operators
        ("_lit1 + _lit2", &|res| match res.matched_exprs.get(&1).unwrap() {
            Expr::Literal(num1) => match res.matched_exprs.get(&2).unwrap() {
                Expr::Literal(num2) => Some(Expr::Literal(num1 + num2)),
                _ => unreachable!()
            },
            _ => unreachable!()
        }),
        ("_lit1 * _lit2", &|res| match res.matched_exprs.get(&1).unwrap() {
            Expr::Literal(num1) => match res.matched_exprs.get(&2).unwrap() {
                Expr::Literal(num2) => Some(Expr::Literal(num1*num2)),
                _ => unreachable!()
            },
            _ => unreachable!()
        }),
        ("_lit1 / _lit2", &|res| match res.matched_exprs.get(&1).unwrap() {
            Expr::Literal(num1) => match res.matched_exprs.get(&2).unwrap() {
                Expr::Literal(num2) => Some(Expr::Literal(num1/num2)),
                _ => unreachable!()
            },
            _ => unreachable!()
        }),
        ("_lit1 ^ _lit2", &|res| match res.matched_exprs.get(&1).unwrap() {
            Expr::Literal(num1) => match res.matched_exprs.get(&2).unwrap() {
                Expr::Literal(num2) => Some(Expr::Literal(num1.powf(*num2))),
                _ => unreachable!()
            },
            _ => unreachable!()
        }),
    ]);
}

pub struct Simplify;

impl ExprVisitor for Simplify {
    fn visit(&mut self, expr: &mut Expr) {
        walk_expr(expr, self);

        *expr = SIMPLIFY_TRANSFORMS.apply_rules(expr);

        // simplify any newly created ast nodes
        walk_expr(expr, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parser::Parser;
    use logos::Logos;

    #[test]
    fn test_constant_fold() {
        let mut expr = Parser::from(Token::lexer("0 + 2 * x")).parse();
        Simplify.visit(&mut expr);

        let expected = Parser::from(Token::lexer("2 * x")).parse();
        assert_eq!(expr, expected);
    }
}
