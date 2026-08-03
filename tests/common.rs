use std::collections::HashMap;

use tinyexpr_rs::parser;
use tinyexpr_rs::errors::ParseError;

/// Parse an expression or panic.
pub fn parse_ok(expr: &str) -> tinyexpr_rs::ast::Expr {
    parser::parse(expr).unwrap_or_else(|e| panic!("{expr:?}: {e}"))
}

/// Evaluate with no variables.
pub fn interp(expr: &str) -> Result<f64, ParseError> {
    let ast = parser::parse(expr)?;
    ast.eval(&HashMap::new())
}

/// Evaluate with variables.
pub fn interp_with(
    expr: &str,
    vars: &HashMap<String, f64>,
) -> Result<f64, ParseError> {
    let ast = parser::parse(expr)?;
    ast.eval(vars)
}

pub fn assert_close(value: f64, expected: f64) {
    let tolerance = 1e-4;

    assert!(
        (value - expected).abs() <= tolerance,
        "expected {expected}, got {value} (diff {})",
        (value - expected).abs()
    );
}

/// Build a variable map.
pub fn vars<const N: usize>(
    pairs: [(&str, f64); N],
) -> HashMap<String, f64> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), *v))
        .collect()
}