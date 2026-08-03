# tinyexpr-rs

A safe, idiomatic Rust port of the original **TinyExpr** mathematical expression parser and evaluator.

Built as part of **Code Resurrection 2026**, this project preserves the grammar and behavior of the original C implementation while redesigning the internals to leverage Rust's ownership model, type system, and modern error handling.

---

# Overview

TinyExpr is a lightweight recursive-descent parser capable of parsing and evaluating mathematical expressions such as

```text
2 + 3 * 4
sqrt(16)
pow(2, 10)
sin(pi / 2)
fac(5)
ncr(5,2)
```

This project reimplements TinyExpr in Rust while preserving the original parser grammar and evaluation semantics wherever practical.

Unlike a direct line-by-line translation, the implementation embraces idiomatic Rust design using enums, pattern matching, ownership, and `Result`-based error handling.

---

# Features

- Recursive-descent parser
- Expression evaluation
- Constant folding optimization
- Built-in mathematical functions
- Variables
- Built-in constants (`pi`, `e`)
- Comprehensive unit and integration tests
- Smoke test suite based on the original TinyExpr tests
- Criterion benchmarks
- Zero `unsafe` Rust
- Modular library API

---

# Supported Operators

| Operator | Meaning |
|----------|---------|
| + | Addition |
| - | Subtraction |
| * | Multiplication |
| / | Division |
| % | Modulo |
| ^ | Power |
| , | Sequence operator |
| +x | Unary plus |
| -x | Unary minus |

---

# Supported Built-in Functions

| Function | Description |
|----------|-------------|
| abs | Absolute value |
| acos | Arc cosine |
| asin | Arc sine |
| atan | Arc tangent |
| atan2 | Two-argument arctangent |
| ceil | Ceiling |
| cos | Cosine |
| cosh | Hyperbolic cosine |
| exp | Exponential |
| fac | Factorial |
| floor | Floor |
| ln | Natural logarithm |
| log | Base-10 logarithm |
| log10 | Base-10 logarithm |
| ncr | Combinations |
| npr | Permutations |
| pi | π constant |
| pow | Power |
| sin | Sine |
| sinh | Hyperbolic sine |
| sqrt | Square root |
| tan | Tangent |
| tanh | Hyperbolic tangent |
| e | Euler's number |

---

# Project Architecture

```text
Source Expression
        │
        ▼
 Lexer (Tokenizer)
        │
        ▼
Recursive Descent Parser
        │
        ▼
 Abstract Syntax Tree (AST)
        │
        ▼
Constant Folding Optimizer
        │
        ▼
 Expression Evaluator
        │
        ▼
       Result
```

---

# Project Structure

```text
src/
├── ast.rs
├── builtins.rs
├── error.rs
├── eval.rs
├── lexer.rs
├── parser.rs
├── token.rs
└── lib.rs

tests/
├── common.rs
├── eval.rs
├── lexer.rs
├── smoke.rs
├── example1.rs
├── example2.rs
└── example3.rs

benches/
└── tinyexpr_bench.rs
```

---

# Grammar

```text
list
    = expr { "," expr }

expr
    = term { ("+" | "-") term }

term
    = factor { ("*" | "/" | "%") factor }

factor
    = power { "^" power }

power
    = { "+" | "-" } base

base
    = number
    | variable
    | function
    | "(" list ")"
```

---

# Example

```rust
use std::collections::HashMap;
use tinyexpr_rs::parser;

fn main() {
    let expr = parser::parse("sqrt(16) + pow(2,3)").unwrap();

    let result = expr.eval(&HashMap::new()).unwrap();

    println!("{result}");
}
```

Output

```text
12
```

---

# Optimization

The optimizer performs constant folding before evaluation.

Example

Input

```text
2 + 3 * 4
```

Original AST

```text
+
├──2
└──*
   ├──3
   └──4
```

Optimized AST

```text
14
```

Constant expressions are evaluated once during optimization, reducing runtime work while preserving semantics.

---

# Compatibility

The parser preserves TinyExpr behavior for

- operator precedence
- left-associative exponentiation
- unary operators
- built-in mathematical functions
- variables
- recursive-descent parsing
- comma sequence operator

---

# Differences from the Original TinyExpr

The following features are intentionally not yet implemented:

- User-defined functions
- Closures
- Scientific notation (`1e3`)
- Parse error source positions

These limitations are documented in the smoke tests.

---

# Why Rust?

The original TinyExpr is implemented in C using manual memory management and raw pointers.

This implementation redesigns those concepts using modern Rust abstractions.

| Original TinyExpr | Rust |
|------------------|------|
| malloc/free | Ownership |
| Raw pointers | Box and references |
| NULL | Option |
| Error codes | Result |
| Tagged integer flags | Enums |
| Manual cleanup | Automatic Drop |

Benefits include

- Memory safety
- No buffer overflows
- No dangling pointers
- Strong type safety
- Automatic resource management
- Improved maintainability

---

# Building

```bash
cargo build
```

---

# Testing

Run all tests

```bash
cargo test
```

Format the project

```bash
cargo fmt
```

Lint the project

```bash
cargo clippy --all-targets --all-features
```

Generate documentation

```bash
cargo doc --no-deps
```

Run benchmarks

```bash
cargo bench
```

---

# Benchmarks

Criterion benchmarks measure

- Simple arithmetic
- Nested expressions
- Function calls
- Variable lookup
- Deep expression trees
- Parser performance
- Evaluator performance

---

# Engineering Decisions

Rather than mechanically translating the original C source, this project was redesigned around idiomatic Rust principles.

Key decisions include

- Strongly typed AST
- Recursive-descent parser
- Pattern matching throughout parsing and evaluation
- Modular lexer/parser/evaluator separation
- Constant folding optimizer
- Comprehensive testing
- Zero `unsafe` code

---

# Current Status

- ✅ Library builds successfully
- ✅ Unit tests passing
- ✅ Smoke tests passing
- ✅ Examples compile
- ✅ Benchmarks implemented
- ✅ Clippy clean (except optional style suggestions)
- ✅ Safe Rust throughout

---

# Future Work

- User-defined functions
- Closure support
- Scientific notation
- Better parser diagnostics
- REPL
- WebAssembly support

---

# Acknowledgements

Based on the original **TinyExpr** library by **Lewis Van Winkle**.

This Rust port was developed as part of the **Code Resurrection 2026** hackathon while preserving the original behavior and adopting modern Rust engineering practices.

---

# License

This project follows the licensing terms of the original TinyExpr project. See the `LICENSE` file for details.