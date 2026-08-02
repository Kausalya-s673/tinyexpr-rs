//! Criterion benchmarks for tinyexpr-rs.
//!
//! Run with:
//!   cargo bench
//!
//! Covers five workload shapes, each measured at both the `parse` stage
//! and the `eval` stage (parse and eval are timed separately so a
//! regression in one doesn't hide in the other's numbers):
//!
//!   - simple arithmetic  — a short, flat expression
//!   - nested expressions — parenthesized, mixed-precedence, moderate depth
//!   - function calls     — several builtin calls, some nested
//!   - deep AST           — two shapes: a long flat operator chain (stresses
//!                          `eval`'s recursive call stack) and deeply nested
//!                          parens (stresses `parse_base`'s recursive descent
//!                          stack specifically)
//!   - variable lookup    — many `Expr::Variable` references against a
//!                          `HashMap`, isolating lookup/hashing cost

use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use tinyexpr_rs::ast::Expr;
use tinyexpr_rs::parser;

// ---- fixture expressions ---------------------------------------------------

const SIMPLE_ARITHMETIC: &str = "2 + 3 * 4 - 1";

const NESTED_EXPRESSIONS: &str = "((2 + 3) * (4 - 1)) / (5 + 2) - ((6 * 2) + (8 / 4))";

const FUNCTION_CALLS: &str = "sqrt(2) + sin(1) * cos(1) - atan2(1, 2) + pow(2, 10) + ln(e)";

/// A long flat chain of additions: `1+1+1+...+1`. `parse_expr`/`parse_term`
/// build this iteratively (no parser recursion per term), but the
/// resulting left-folded `Binary` tree still has `eval`-time recursion
/// depth proportional to `terms` — good for isolating eval stack cost from
/// parse cost.
fn deep_chain(terms: usize) -> String {
    std::iter::repeat("1")
        .take(terms)
        .collect::<Vec<_>>()
        .join("+")
}

/// Deeply nested parens: `(((...(1)...)))`. Every `(` forces
/// `parse_base` -> `parse_list` -> ... -> `parse_base` to recurse one
/// level deeper, so this specifically stresses the *parser's* call stack
/// rather than eval's.
fn deep_nested_parens(depth: usize) -> String {
    let mut s = String::with_capacity(depth * 2 + 1);
    s.extend(std::iter::repeat('(').take(depth));
    s.push('1');
    s.extend(std::iter::repeat(')').take(depth));
    s
}

const DEEP_CHAIN_TERMS: usize = 1_000;
const DEEP_NESTING_DEPTH: usize = 500;

/// Many references to a handful of bound variables, forcing repeated
/// `HashMap` lookups at eval time.
fn variable_lookup_expr(repeats: usize) -> String {
    std::iter::repeat("a + b + c - d")
        .take(repeats)
        .collect::<Vec<_>>()
        .join("+")
}

const VARIABLE_LOOKUP_REPEATS: usize = 250;

fn sample_vars() -> HashMap<String, f64> {
    let mut vars = HashMap::new();
    vars.insert("a".to_string(), 1.0);
    vars.insert("b".to_string(), 2.0);
    vars.insert("c".to_string(), 3.0);
    vars.insert("d".to_string(), 4.0);
    vars
}

// ---- parse benchmarks -------------------------------------------------------

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

    let var_lookup_src = variable_lookup_expr(VARIABLE_LOOKUP_REPEATS);
    group.bench_function("variable_lookup", |b| {
        b.iter(|| parser::parse(black_box(&var_lookup_src)))
    });

    group.finish();
}

// ---- eval benchmarks ---------------------------------------------------------

/// Parses `src` once up front (outside the timed section) so each
/// benchmark isolates `eval` cost from `parse` cost.
fn parsed(src: &str) -> Expr {
    parser::parse(src).unwrap_or_else(|e| panic!("failed to parse {src:?}: {e}"))
}

fn bench_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval");
    let empty_vars: HashMap<String, f64> = HashMap::new();

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
    let var_lookup_ast = parsed(&variable_lookup_expr(VARIABLE_LOOKUP_REPEATS));
    group.bench_function("variable_lookup", |b| {
        b.iter(|| var_lookup_ast.eval(black_box(&vars)))
    });

    group.finish();
}

criterion_group!(benches, bench_parse, bench_eval);
criterion_main!(benches);
