//! Criterion benchmarks for tinyexpr-rs.
//!
//! Run with:
//!
//! ```bash
//! cargo bench
//! ```
//!
//! Covers five workload shapes, each measured at both the parsing and
//! evaluation stages so regressions in one phase don't hide in the other.
//!
//! - Simple arithmetic — a short, flat expression.
//! - Nested expressions — parenthesized, mixed-precedence expressions.
//! - Function calls — several builtin calls, including nested calls.
//! - Deep AST — two stress cases:
//!     - a long flat operator chain (stresses `eval` recursion depth)
//!     - deeply nested parentheses (stresses parser recursion depth)
//! - Variable lookup — many `Expr::Variable` references against a
//!     `HashMap`, isolating lookup/hashing cost.

use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;

use tinyexpr_rs::ast::Expr;
use tinyexpr_rs::parser;

// -----------------------------------------------------------------------------
// Benchmark fixtures
// -----------------------------------------------------------------------------

const SIMPLE_ARITHMETIC: &str = "2 + 3 * 4 - 1";

const NESTED_EXPRESSIONS: &str = "((2 + 3) * (4 - 1)) / (5 + 2) - ((6 * 2) + (8 / 4))";

const FUNCTION_CALLS: &str = "sqrt(2) + sin(1) * cos(1) - atan2(1, 2) + pow(2, 10) + ln(e)";

const DEEP_CHAIN_TERMS: usize = 1_000;
const DEEP_NESTING_DEPTH: usize = 500;
const VARIABLE_LOOKUP_REPEATS: usize = 250;

/// Generates a long flat addition chain:
///
/// `1+1+1+...+1`
fn deep_chain(terms: usize) -> String {
    std::iter::repeat_n("1", terms)
        .collect::<Vec<_>>()
        .join("+")
}

/// Generates deeply nested parentheses:
///
/// `((((1))))`
fn deep_nested_parens(depth: usize) -> String {
    let mut s = String::with_capacity(depth * 2 + 1);

    s.extend(std::iter::repeat_n('(', depth));
    s.push('1');
    s.extend(std::iter::repeat_n(')', depth));

    s
}

/// Generates repeated variable expressions.
fn variable_lookup_expr(repeats: usize) -> String {
    std::iter::repeat_n("a + b + c - d", repeats)
        .collect::<Vec<_>>()
        .join("+")
}

fn sample_vars() -> HashMap<String, f64> {
    let mut vars = HashMap::new();
    vars.insert("a".into(), 1.0);
    vars.insert("b".into(), 2.0);
    vars.insert("c".into(), 3.0);
    vars.insert("d".into(), 4.0);
    vars
}

// -----------------------------------------------------------------------------
// Parse benchmarks
// -----------------------------------------------------------------------------

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("simple_arithmetic", |b| {
        b.iter(|| parser::parse(black_box(SIMPLE_ARITHMETIC)))
    });

    group.bench_function("nested_expressions", |b| {
        b.iter(|| parser::parse(black_box(NESTED_EXPRESSIONS)))
    });

    group.bench_function("function_calls", |b| {
        b.iter(|| parser::parse(black_box(FUNCTION_CALLS)))
    });

    let deep_chain_src = deep_chain(DEEP_CHAIN_TERMS);

    group.bench_function("deep_ast_flat_chain", |b| {
        b.iter(|| parser::parse(black_box(&deep_chain_src)))
    });

    let deep_parens_src = deep_nested_parens(DEEP_NESTING_DEPTH);

    group.bench_function("deep_ast_nested_parens", |b| {
        b.iter(|| parser::parse(black_box(&deep_parens_src)))
    });

    let variable_src = variable_lookup_expr(VARIABLE_LOOKUP_REPEATS);

    group.bench_function("variable_lookup", |b| {
        b.iter(|| parser::parse(black_box(&variable_src)))
    });

    group.finish();
}

// -----------------------------------------------------------------------------
// Eval benchmarks
// -----------------------------------------------------------------------------

/// Parses once outside the benchmark so only evaluation time is measured.
fn parsed(src: &str) -> Expr {
    parser::parse(src).unwrap_or_else(|e| panic!("failed to parse {src:?}: {e}"))
}

fn bench_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval");

    let empty_vars = HashMap::<String, f64>::new();

    let simple = parsed(SIMPLE_ARITHMETIC);
    group.bench_function("simple_arithmetic", |b| {
        b.iter(|| simple.eval(black_box(&empty_vars)))
    });

    let nested = parsed(NESTED_EXPRESSIONS);
    group.bench_function("nested_expressions", |b| {
        b.iter(|| nested.eval(black_box(&empty_vars)))
    });

    let functions = parsed(FUNCTION_CALLS);
    group.bench_function("function_calls", |b| {
        b.iter(|| functions.eval(black_box(&empty_vars)))
    });

    let deep_chain_ast = parsed(&deep_chain(DEEP_CHAIN_TERMS));

    group.bench_function("deep_ast_flat_chain", |b| {
        b.iter(|| deep_chain_ast.eval(black_box(&empty_vars)))
    });

    let deep_parens_ast = parsed(&deep_nested_parens(DEEP_NESTING_DEPTH));

    group.bench_function("deep_ast_nested_parens", |b| {
        b.iter(|| deep_parens_ast.eval(black_box(&empty_vars)))
    });

    let vars = sample_vars();
    let variable_ast = parsed(&variable_lookup_expr(VARIABLE_LOOKUP_REPEATS));

    group.bench_function("variable_lookup", |b| {
        b.iter(|| variable_ast.eval(black_box(&vars)))
    });

    group.finish();
}

criterion_group!(benches, bench_parse, bench_eval);
criterion_main!(benches);
