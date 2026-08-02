//! Evaluates a parsed `Expr` tree to an `f64`.

use crate::ast::{BinaryOp, Builtin, Expr, UnaryOp};
use crate::builtins;
use crate::errors::ParseError;
use std::collections::HashMap;

impl Expr {
    /// Evaluates this expression to a single `f64`, resolving `Variable`
    /// nodes against `vars`.
    ///
    /// Mirrors upstream TinyExpr's evaluation semantics: domain errors
    /// (`sqrt(-1)`, `asin(2)`, ...) and division by zero are **not**
    /// treated as Rust-level errors. Like the underlying C library, they
    /// simply produce `NaN` / `±INFINITY` per IEEE 754 and propagate
    /// through the rest of the expression exactly like any other float.
    /// The only case that produces an `Err` here is an unbound
    /// `Variable` — that's a genuine "cannot evaluate this" condition,
    /// not a numeric edge case.
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, ParseError> {
        match self {
            Expr::Number(n) => Ok(*n),

            Expr::Variable(name) => vars
                .get(name)
                .copied()
                .ok_or_else(|| ParseError::UnknownIdentifier(name.clone())),

            Expr::Unary { op, expr } => {
                let value = expr.eval(vars)?;
                Ok(match op {
                    UnaryOp::Plus => value,
                    UnaryOp::Minus => -value,
                })
            }

            Expr::Binary { left, op, right } => {
                let l = left.eval(vars)?;
                let r = right.eval(vars)?;
                Ok(match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    // Follows IEEE 754 (±INFINITY, or NaN for 0/0) — same
                    // as the C `/` TinyExpr compiles down to; not an error.
                    BinaryOp::Div => l / r,
                    // Rust's `%` on f64 is `fmod`-equivalent (result takes
                    // the sign of `l`), matching the C `fmod` TinyExpr
                    // uses for `%`.
                    BinaryOp::Mod => l % r,
                    BinaryOp::Pow => l.powf(r),
                })
            }

            Expr::Function { function, args } => eval_function(function, args, vars),

            Expr::Sequence(exprs) => {
                // Comma operator: every element is evaluated in order (so
                // an error or side-effect-bearing computation in an
                // earlier element still surfaces), but only the value of
                // the last element is kept.
                debug_assert!(
                    exprs.len() >= 2,
                    "Expr::Sequence should always hold at least 2 elements"
                );
                let mut result = 0.0;
                for expr in exprs {
                    result = expr.eval(vars)?;
                }
                Ok(result)
            }
        }
    }
}

fn eval_function(
    function: &Builtin,
    args: &[Expr],
    vars: &HashMap<String, f64>,
) -> Result<f64, ParseError> {
    // Defensive check: a hand-built `Expr::Function` (bypassing the
    // parser — e.g. constructed directly in a test) could carry the wrong
    // number of args, which would otherwise panic on the `arg(i)` indexing
    // below. The parser itself already guarantees this holds for anything
    // it produces.
    let expected = builtins::arity(function.clone());
    if args.len() != expected {
        return Err(ParseError::ArityMismatch {
            expected,
            found: args.len(),
        });
    }

    // Evaluates the nth argument; safe to index directly now that the
    // arity check above has run.
    let arg = |i: usize| args[i].eval(vars);

    Ok(match function {
        Builtin::Abs => arg(0)?.abs(),
        Builtin::Acos => arg(0)?.acos(),
        Builtin::Asin => arg(0)?.asin(),
        Builtin::Atan => arg(0)?.atan(),
        Builtin::Atan2 => arg(0)?.atan2(arg(1)?),
        Builtin::Ceil => arg(0)?.ceil(),
        Builtin::Cos => arg(0)?.cos(),
        Builtin::Cosh => arg(0)?.cosh(),
        Builtin::E => std::f64::consts::E,
        Builtin::Exp => arg(0)?.exp(),
        Builtin::Fac => factorial(arg(0)?),
        Builtin::Floor => arg(0)?.floor(),
        Builtin::Ln => arg(0)?.ln(),
        // Upstream TinyExpr's default build (without the TE_NAT_LOG
        // compile flag) maps `log` to base-10 log, *not* natural log —
        // `ln` is the natural log. Confirmed by upstream's own reference
        // tests: `log(10) == 1.0`, `log10(10) == 1.0`, `ln(e) == 1.0`.
        Builtin::Log => arg(0)?.log10(),
        Builtin::Log10 => arg(0)?.log10(),
        Builtin::Ncr => ncr(arg(0)?, arg(1)?),
        Builtin::Npr => npr(arg(0)?, arg(1)?),
        Builtin::Pi => std::f64::consts::PI,
        Builtin::Pow => arg(0)?.powf(arg(1)?),
        Builtin::Sin => arg(0)?.sin(),
        Builtin::Sinh => arg(0)?.sinh(),
        Builtin::Sqrt => arg(0)?.sqrt(),
        Builtin::Tan => arg(0)?.tan(),
        Builtin::Tanh => arg(0)?.tanh(),
    })
}

/// `n!`. `NaN` for negative or non-integer input — consistent with how
/// every other domain error in this module is handled (invalid input
/// yields `NaN` rather than a Rust `Err`).
///
/// Note: this walks a plain `f64` product loop rather than replicating
/// upstream's `unsigned long` overflow bookkeeping bit-for-bit, so very
/// large inputs will differ in exactly *when* they roll over to
/// `INFINITY` (native f64 overflow vs. C's integer-overflow guard) — both
/// still end up at `INFINITY`, just not necessarily at the same `n`.
fn factorial(n: f64) -> f64 {
    if n < 0.0 || n.fract() != 0.0 {
        return f64::NAN;
    }
    let mut result = 1.0_f64;
    let mut i = 1.0_f64;
    while i <= n {
        result *= i;
        i += 1.0;
    }
    result
}

/// `n choose r`.
fn ncr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
        return f64::NAN;
    }
    let r = r.min(n - r);
    let mut result = 1.0_f64;
    let mut i = 0.0_f64;
    while i < r {
        result *= (n - i) / (i + 1.0);
        i += 1.0;
    }
    result.round()
}

/// `n permute r`.
fn npr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
        return f64::NAN;
    }
    let mut result = 1.0_f64;
    let mut i = 0.0_f64;
    while i < r {
        result *= n - i;
        i += 1.0;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn eval_str(src: &str) -> f64 {
        eval_with(src, &HashMap::new())
    }

    fn eval_with(src: &str, vars: &HashMap<String, f64>) -> f64 {
        parser::parse(src)
            .unwrap_or_else(|e| panic!("parse error for {src:?}: {e}"))
            .eval(vars)
            .unwrap_or_else(|e| panic!("eval error for {src:?}: {e}"))
    }

    #[test]
    fn number_literal() {
        assert_eq!(eval_str("42"), 42.0);
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(eval_str("2+2*2"), 6.0);
        assert_eq!(eval_str("(2+2)*2"), 8.0);
        assert_eq!(eval_str("2*2+2"), 6.0);
    }

    #[test]
    fn modulo() {
        assert_eq!(eval_str("5%2"), 1.0);
    }

    #[test]
    fn power_is_left_associative_by_default() {
        // TinyExpr's documented default: a^b^c == (a^b)^c, spreadsheet-style.
        assert_eq!(eval_str("5^2"), 25.0);
        assert_eq!(eval_str("2^3^2"), 64.0); // (2^3)^2 = 8^2 = 64
    }

    #[test]
    fn unary_sign_binds_before_power_by_default() {
        // TinyExpr's documented default: -a^b == (-a)^b, not -(a^b).
        assert_eq!(eval_str("-2^2"), 4.0);
        assert_eq!(eval_str("-5"), -5.0);
        assert_eq!(eval_str("+5"), 5.0);
        assert_eq!(eval_str("--5"), 5.0);
    }

    #[test]
    fn division_by_zero_is_infinity_not_an_error() {
        assert_eq!(eval_str("1/0"), f64::INFINITY);
        assert_eq!(eval_str("-1/0"), f64::NEG_INFINITY);
        assert!(eval_str("0/0").is_nan());
    }

    #[test]
    fn variable_lookup() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 3.0);
        vars.insert("y".to_string(), 4.0);
        assert_eq!(eval_with("x*x+y*y", &vars), 25.0);
    }

    #[test]
    fn unbound_variable_errors() {
        let result = parser::parse("x").unwrap().eval(&HashMap::new());
        assert!(matches!(result, Err(ParseError::UnknownIdentifier(name)) if name == "x"));
    }

    #[test]
    fn constants() {
        assert_eq!(eval_str("pi"), std::f64::consts::PI);
        assert_eq!(eval_str("e"), std::f64::consts::E);
    }

    #[test]
    fn one_arg_functions_with_and_without_parens() {
        assert_eq!(eval_str("sqrt(16)"), 4.0);
        assert_eq!(eval_str("sqrt 16"), 4.0);
        assert_eq!(eval_str("abs(-5)"), 5.0);
        assert_eq!(eval_str("sin 0"), 0.0);
    }

    #[test]
    fn logs() {
        // TinyExpr's default (no TE_NAT_LOG): `log` == `log10`, `ln` is natural.
        assert_eq!(eval_str("log(10)"), 1.0);
        assert_eq!(eval_str("log10(10)"), 1.0);
        assert_eq!(eval_str("ln(e)"), 1.0);
    }

    #[test]
    fn two_arg_functions() {
        assert_eq!(eval_str("pow(2,3)"), 8.0);
        assert_eq!(eval_str("atan2(0,1)"), 0.0);
    }

    #[test]
    fn factorial_ncr_npr() {
        assert_eq!(eval_str("fac(5)"), 120.0);
        assert_eq!(eval_str("ncr(5,2)"), 10.0);
        assert_eq!(eval_str("npr(5,2)"), 20.0);
        assert!(eval_str("fac(-1)").is_nan());
    }

    #[test]
    fn sequence_evaluates_all_keeps_last() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 1.0);
        assert_eq!(eval_with("1,2,3", &vars), 3.0);
        assert_eq!(eval_with("x, x+1, x+2", &vars), 3.0);
    }

    #[test]
    fn sequence_surfaces_errors_from_earlier_elements() {
        // Only the last element's value is kept, but an error in an
        // earlier element must still propagate rather than being silently
        // skipped.
        let result = parser::parse("bogus, 1").unwrap().eval(&HashMap::new());
        assert!(matches!(result, Err(ParseError::UnknownIdentifier(_))));
    }

    #[test]
    fn domain_errors_yield_nan_not_err() {
        assert!(eval_str("sqrt(-1)").is_nan());
        assert!(eval_str("asin(2)").is_nan());
    }

    #[test]
    fn hand_built_arity_mismatch_does_not_panic() {
        // Bypasses the parser entirely to exercise the defensive arity
        // check in eval_function.
        let bad = Expr::Function {
            function: Builtin::Pow, // arity 2
            args: vec![Expr::Number(2.0)],
        };
        let result = bad.eval(&HashMap::new());
        assert!(matches!(
            result,
            Err(ParseError::ArityMismatch { expected: 2, found: 1 })
        ));
    }
}