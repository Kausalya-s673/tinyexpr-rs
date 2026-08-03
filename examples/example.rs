//! Equivalent of TinyExpr's original `example.c`.
//!
//! Run with:
//! cargo run --example example

use std::collections::HashMap;
use tinyexpr_rs::parser;

fn main() {
    let expression = "sqrt(5^2+7^2+11^2+(8-2)^2)";

    // Parse the expression into an AST.
    let ast = parser::parse(expression)
        .unwrap_or_else(|e| panic!("Parse error: {e}"));

    // No variables are needed for this expression.
    let vars = HashMap::new();

    // Evaluate the AST.
    let result = ast
        .eval(&vars)
        .unwrap_or_else(|e| panic!("Evaluation error: {e}"));

    println!("Expression: {expression}");
    println!("Result: {result}");
}