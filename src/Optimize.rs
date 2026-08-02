//! Constant folding for the tinyexpr-rs AST.
//!
//! `Expr::optimize` walks the tree bottom-up and collapses any subtree
//! whose value doesn't depend on a variable binding into a single
//! `Expr::Number`. It deliberately does **not** duplicate any arithmetic
//! or builtin-function logic — folding works by constructing the
//! already-partially-optimized node and running it through the existing
//! `Expr::eval` with an empty variable map. That keeps constant folding
//! permanently in sync with actual evaluation semantics (left-associative
//! `^`, `-a^b == (-a)^b`, `log` == `log10`, IEEE-754 NaN/Infinity
//! handling, etc.) with zero risk of the two drifting apart.

use crate::ast::Expr;
use std::collections::HashMap;

impl Expr {
    /// Recursively constant-folds this expression, returning a new,
    /// optimized `Expr`.
    ///
    /// - `Number` and `Variable` are returned unchanged — variables are
    ///   never folded, since their value isn't known until `eval` is
    ///   called with a variable map.
    /// - `Unary`, `Binary`, and `Function` nodes have their operands
    ///   optimized first; if *all* of a node's immediate operands turned
    ///   out to be constants, the node itself is folded into a single
    ///   `Number`. Otherwise the node is kept, but with its
    ///   (possibly-now-partially-constant) operands in place.
    /// - `Sequence` always keeps every element, in order. Even though a
    ///   pure numeric element has no effect on the final value beyond the
    ///   last one, an earlier element can still be an unbound `Variable`
    ///   or a mistyped `Function` call that fails at eval time — dropping
    ///   it would silently suppress that error. Each element is still
    ///   individually optimized.
    pub fn optimize(self) -> Expr {
        match self {
            Expr::Number(_) | Expr::Variable(_) => self,

            Expr::Unary { op, expr } => {
                let expr = expr.optimize();
                fold_if_constant(Expr::Unary {
                    op,
                    expr: Box::new(expr),
                })
            }

            Expr::Binary { left, op, right } => {
                let left = left.optimize();
                let right = right.optimize();
                fold_if_constant(Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }

            Expr::Function { function, args } => {
                let args: Vec<Expr> = args.into_iter().map(Expr::optimize).collect();
                fold_if_constant(Expr::Function { function, args })
            }

            Expr::Sequence(exprs) => {
                Expr::Sequence(exprs.into_iter().map(Expr::optimize).collect())
            }
        }
    }
}

/// If every immediate operand of `node` is already a constant
/// (`Expr::Number`), evaluates `node` (with no variable bindings, since
/// none are needed) and returns the result as `Expr::Number`. Otherwise
/// returns `node` unchanged.
///
/// `node` is assumed to be an `Unary`, `Binary`, or `Function` — its
/// operands have already been through `optimize()` by the caller, so if
/// an operand isn't `Expr::Number` at this point, it genuinely depends on
/// a variable somewhere in its subtree and can't be folded further.
fn fold_if_constant(node: Expr) -> Expr {
    let all_operands_constant = match &node {
        Expr::Unary { expr, .. } => matches!(**expr, Expr::Number(_)),
        Expr::Binary { left, right, .. } => {
            matches!(**left, Expr::Number(_)) && matches!(**right, Expr::Number(_))
        }
        Expr::Function { args, .. } => args.iter().all(|a| matches!(a, Expr::Number(_))),
        Expr::Number(_) | Expr::Variable(_) | Expr::Sequence(_) => false,
    };

    if all_operands_constant {
        // No variables are involved (just checked above), so `eval` can
        // only fail here on a hand-built arity mismatch bypassing the
        // parser — in that case we leave the node unfolded rather than
        // losing it. Domain errors (e.g. `sqrt(-1)`) are not failures:
        // they fold to `Expr::Number(NaN)`, matching eval-time behavior.
        if let Ok(value) = node.eval(&HashMap::new()) {
            return Expr::Number(value);
        }
    }

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Builtin, UnaryOp};
    use crate::parser;

    fn optimized(src: &str) -> Expr {
        parser::parse(src).unwrap().optimize()
    }

    #[test]
    fn folds_arithmetic() {
        assert_eq!(optimized("2+3*4"), Expr::Number(14.0));
        assert_eq!(optimized("(2+3)*4"), Expr::Number(20.0));
        assert_eq!(optimized("10%3"), Expr::Number(1.0));
    }

    #[test]
    fn folds_power_left_associatively() {
        // Must match eval's documented default: (2^3)^2 == 64, not 512.
        assert_eq!(optimized("2^3^2"), Expr::Number(64.0));
    }

    #[test]
    fn folds_unary() {
        assert_eq!(optimized("-5"), Expr::Number(-5.0));
        assert_eq!(optimized("+5"), Expr::Number(5.0));
        assert_eq!(optimized("--5"), Expr::Number(5.0));
        // Sign binds before ^ by default: -2^2 == (-2)^2 == 4.
        assert_eq!(optimized("-2^2"), Expr::Number(4.0));
    }

    #[test]
    fn folds_builtin_functions() {
        assert_eq!(optimized("sqrt(16)"), Expr::Number(4.0));
        assert_eq!(optimized("pow(2,3)"), Expr::Number(8.0));
        assert_eq!(optimized("pi"), Expr::Number(std::f64::consts::PI));
        assert_eq!(optimized("fac(5)"), Expr::Number(120.0));
    }

    #[test]
    fn folds_domain_errors_to_nan_not_a_failure() {
        match optimized("sqrt(-1)") {
            Expr::Number(n) => assert!(n.is_nan()),
            other => panic!("expected a folded NaN constant, got {other:?}"),
        }
    }

    #[test]
    fn preserves_bare_variable() {
        assert_eq!(optimized("x"), Expr::Variable("x".to_string()));
    }

    #[test]
    fn does_not_fold_across_a_variable() {
        // The whole expression can't become a Number, but the constant
        // sub-expression (2*3) inside it still should.
        let result = optimized("x + (2*3)");
        assert_eq!(
            result,
            Expr::Binary {
                left: Box::new(Expr::Variable("x".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expr::Number(6.0)),
            }
        );
    }

    #[test]
    fn partially_folds_function_args() {
        // atan2's second argument is constant-foldable even though the
        // whole call can't be, because the first argument is a variable.
        let result = optimized("atan2(x, 1+1)");
        assert_eq!(
            result,
            Expr::Function {
                function: Builtin::Atan2,
                args: vec![Expr::Variable("x".to_string()), Expr::Number(2.0)],
            }
        );
    }

    #[test]
    fn folds_nested_unary_around_variable_stays_unfolded() {
        let result = optimized("-x");
        assert_eq!(
            result,
            Expr::Unary {
                op: UnaryOp::Minus,
                expr: Box::new(Expr::Variable("x".to_string())),
            }
        );
    }

    #[test]
    fn sequence_keeps_every_element_and_order() {
        let result = optimized("1+1, x, 2+2");
        assert_eq!(
            result,
            Expr::Sequence(vec![
                Expr::Number(2.0),
                Expr::Variable("x".to_string()),
                Expr::Number(4.0),
            ])
        );
    }

    #[test]
    fn sequence_does_not_swallow_an_earlier_error() {
        // "bogus, 5" would fold to just `5` if we collapsed the sequence,
        // silently hiding the UnknownIdentifier error `bogus` should
        // still raise at eval time. Structure must be preserved.
        let result = optimized("bogus, 5");
        assert_eq!(
            result,
            Expr::Sequence(vec![Expr::Variable("bogus".to_string()), Expr::Number(5.0),])
        );

        use crate::errors::ParseError;
        let eval_result = result.eval(&HashMap::new());
        assert!(matches!(eval_result, Err(ParseError::UnknownIdentifier(_))));
    }

    #[test]
    fn optimized_and_unoptimized_agree_on_value() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);
        vars.insert("y".to_string(), 4.0);

        for src in [
            "2+2*2",
            "x*x+y*y",
            "-2^2",
            "sqrt(x*x+y*y)",
            "atan2(y, x) + pi",
            "1,2,x+3",
        ] {
            let original = parser::parse(src).unwrap().eval(&vars).unwrap();
            let optimized = parser::parse(src).unwrap().optimize().eval(&vars).unwrap();
            assert_eq!(original, optimized, "mismatch for {src:?}");
        }
    }
}
