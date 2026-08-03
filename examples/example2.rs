//! Equivalent of TinyExpr's `example2.c`.
//!
//! Parses an expression once, then evaluates it multiple times with
//! different variable bindings.
//!
//! Run with:
//! cargo run --example example2

use std::collections::HashMap;
use tinyexpr_rs::parser;

fn main() {
    let expr = parser::parse("x*x+y*y").unwrap_or_else(|e| panic!("Parse error: {e:?}"));

    let mut vars = HashMap::new();

    vars.insert("x".to_string(), 3.0);
    vars.insert("y".to_string(), 4.0);

    let value = expr.eval(&vars).unwrap();
    println!("x=3, y=4  -> {}", value);

    vars.insert("x".to_string(), 5.0);
    vars.insert("y".to_string(), 12.0);

    let value = expr.eval(&vars).unwrap();
    println!("x=5, y=12 -> {}", value);
}
