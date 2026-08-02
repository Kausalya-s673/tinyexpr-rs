//! Port of `example2.c`: parse an expression once, then evaluate it
//! multiple times against different variable bindings — demonstrating
//! that binding happens at eval-time, not at parse-time, and that a
//! parsed `Expr` can be reused across evaluations.
//!
//! Upstream reads the expression from `argv[1]`; a fixed expression is
//! used here since integration tests don't take CLI input, but the
//! "compile once, bind `x`/`y` and eval as many times as you like" shape
//! is the same.

mod common;
use common::*;

#[test]
fn compile_once_eval_with_different_variables() {
    let expr = parse_ok("x*x+y*y");

    let first = expr.eval(&vars([("x", 3.0), ("y", 4.0)])).unwrap();
    assert_close(first, 25.0);

    // Same parsed expression, reused with entirely different bindings —
    // matches upstream's point that `x`/`y` "can be changed here, and
    // eval can be called as many times as you like."
    let second = expr.eval(&vars([("x", 5.0), ("y", 12.0)])).unwrap();
    assert_close(second, 169.0);
}

#[test]
fn reports_an_error_for_a_malformed_expression() {
    // Upstream's `Usage:` / error-position-printing branch when
    // `te_compile` fails. This port has no error position (see
    // smoke.rs's module docs), so only the overall failure is checked.
    assert!(interp_with("x*x+", &vars([("x", 1.0)])).is_err());
}