//! Shared helpers for the ported TinyExpr integration tests.
//!
//! NOTE ON THE CRATE NAME: these files assume your `Cargo.toml` package
//! (and therefore library) is named `tinyexpr` (matching upstream's name
//! and the pre-existing `tinyexpr-rs`/`tinyexpr` crates on crates.io). If
//! your `[package] name` is different, update the `use tinyexpr::...`
//! lines at the top of each test file accordingly — that's the only
//! place the name is assumed.

#![allow(dead_code)]

use std::collections::HashMap;
use tinyexpr::parser;

// Re-exported (not just `use`d) so that `use common::*;` in each test
// file brings these into scope too, since several tests need to name
// `ParseError`'s variants or `Expr`'s directly.
pub use tinyexpr::ast::Expr;
pub use tinyexpr::errors::ParseError;

/// Absolute tolerance mirroring upstream TinyExpr's `minctest` `lfequal`
/// macro, which the original C suite (`smoke.c`) uses for every
/// floating-point comparison. Empirically ~1e-4: several of the suite's
/// own hardcoded expected values are truncated to 4-5 decimal digits
/// (`pi` as `3.14159`, `atan2(2,1)` as `1.1071`), so anything tighter
/// than this would reject those literals even though the implementation
/// is correct.
pub const EPS: f64 = 1e-4;

/// Asserts `actual` and `expected` agree to within [`EPS`].
pub fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= EPS,
        "expected {expected}, got {actual} (diff {})",
        (actual - expected).abs()
    );
}

/// Parses and evaluates `src` with no bound variables — the Rust
/// equivalent of upstream's `te_interp(expr, &err)` for expressions that
/// don't reference any free variable.
pub fn interp(src: &str) -> Result<f64, ParseError> {
    interp_with(src, &HashMap::new())
}

/// Parses and evaluates `src` against a supplied variable map.
///
/// Note this collapses two upstream C stages (`te_compile` validating
/// identifiers against a fixed `te_variable` list, then `te_eval`) into
/// one: this port's `parser::parse` never checks whether an identifier
/// is bound — an unrecognized name always parses fine as `Expr::Variable`
/// and only fails at `eval` time (`ParseError::UnknownIdentifier`) if
/// `vars` doesn't contain it. Several ported tests below call this out
/// explicitly where it changes which stage an error surfaces at.
pub fn interp_with(src: &str, vars: &HashMap<String, f64>) -> Result<f64, ParseError> {
    let expr = parser::parse(src)?;
    expr.eval(vars)
}

/// Convenience for building a variable map inline, e.g.
/// `vars([("x", 3.0), ("y", 4.0)])`.
pub fn vars(pairs: impl IntoIterator<Item = (&'static str, f64)>) -> HashMap<String, f64> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Parses `src` and returns the resulting `Expr`, panicking with a
/// helpful message on parse failure — for tests that want to inspect the
/// tree itself (e.g. the optimizer tests) rather than just the final
/// value.
pub fn parse_ok(src: &str) -> Expr {
    parser::parse(src).unwrap_or_else(|e| panic!("parse error for {src:?}: {e}"))
}