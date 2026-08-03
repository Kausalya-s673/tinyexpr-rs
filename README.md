# tinyexpr-rs

A safe, idiomatic Rust port of the original **TinyExpr** mathematical expression parser and evaluator.

Built as part of **Code Resurrection 2026**, this project preserves the grammar and behavior of the original C implementation while redesigning the internals to leverage Rust's ownership model, type system, and modern error handling.

---

## Overview

TinyExpr is a lightweight recursive-descent parser capable of parsing and evaluating mathematical expressions such as:

```text
2 + 3 * 4
sqrt(16)
pow(2, 10)
sin(pi / 2)
```

This project reimplements TinyExpr in Rust while maintaining behavioral compatibility with the original implementation.

Unlike a direct line-by-line translation, this implementation embraces idiomatic Rust design principles, resulting in safer memory management, clearer abstractions, and improved maintainability.

---

# Features

* Recursive-descent parser
* Expression evaluation
* Constant folding optimization
* Built-in mathematical functions
* Variable support
* Comprehensive unit tests
* Criterion benchmarks
* Continuous Integration
* Zero unsafe Rust

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
├── ast.rs          # Abstract Syntax Tree definitions
├── builtins.rs     # Built-in mathematical functions
├── error.rs        # Parser and evaluation errors
├── eval.rs         # Expression evaluation
├── lexer.rs        # Tokenizer
├── parser.rs       # Recursive-descent parser
├── token.rs        # Token definitions
└── lib.rs          # Public API

tests/
├── lexer.rs
├── parser.rs
├── eval.rs

benches/
└── benchmark.rs
```

---

# Grammar

The parser preserves TinyExpr's original grammar.

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
use tinyexpr_rs::eval;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = eval("2 + 3 * 4")?;

    println!("{result}");

    Ok(())
}
```

Output

```text
14
```

---

# Supported Built-in Functions

| Function | Description             |
| -------- | ----------------------- |
| abs      | Absolute value          |
| acos     | Arc cosine              |
| asin     | Arc sine                |
| atan     | Arc tangent             |
| atan2    | Two-argument arctangent |
| ceil     | Ceiling                 |
| cos      | Cosine                  |
| cosh     | Hyperbolic cosine       |
| exp      | Exponential             |
| fac      | Factorial               |
| floor    | Floor                   |
| ln       | Natural logarithm       |
| log      | Base-10 logarithm       |
| log10    | Base-10 logarithm       |
| ncr      | Combinations            |
| npr      | Permutations            |
| pi       | π                       |
| pow      | Power                   |
| sin      | Sine                    |
| sinh     | Hyperbolic sine         |
| sqrt     | Square root             |
| tan      | Tangent                 |
| tanh     | Hyperbolic tangent      |
| e        | Euler's number          |

---

# Optimization

The optimizer performs constant folding before evaluation.

Example:

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

This reduces runtime work while preserving expression semantics.

---

# Compatibility

This project preserves the behavior of the original TinyExpr implementation, including:

* operator precedence
* associativity
* unary operators
* mathematical functions
* variables
* recursive-descent parsing strategy

---

# Why Rust?

The original TinyExpr is implemented in C using manual memory management, tagged integer flags, and raw pointers.

This implementation redesigns those concepts using modern Rust abstractions.

| Original C           | Rust                        |
| -------------------- | --------------------------- |
| malloc/free          | Ownership + Drop            |
| Raw pointers         | Box and references          |
| NULL                 | Option                      |
| Error codes          | Result                      |
| Tagged integer flags | Enums                       |
| Manual cleanup       | Automatic memory management |

Benefits include:

* Memory safety
* No buffer overflows
* No dangling pointers
* No manual memory management
* Strong type safety
* Improved maintainability

---

# Testing

Run all tests:

```bash
cargo test
```

Format the project:

```bash
cargo fmt --check
```

Lint with Clippy:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Generate documentation:

```bash
cargo doc --no-deps
```

Run benchmarks:

```bash
cargo bench
```

---

# Performance

Benchmarks are implemented using Criterion and compare common expression workloads, including:

* arithmetic expressions
* nested expressions
* function evaluation
* variable lookup
* deep expression trees

---

# Engineering Decisions

Rather than mechanically translating the C source, this project was re-architected to embrace idiomatic Rust.

Key design decisions include:

* Recursive-descent parser implemented using enums and pattern matching
* Strongly typed Abstract Syntax Tree
* Structured error handling with `Result`
* Separation of lexer, parser, optimizer, and evaluator
* Safe ownership model without unsafe code
* Comprehensive automated testing and benchmarking

---

# Future Improvements

* Improved parser diagnostics with source spans
* Expression simplification rules
* User-defined functions
* Additional mathematical operators
* REPL application
* WASM support

---

# Acknowledgements

This project is based on the original **TinyExpr** library by Lewis Van Winkle.

It was reimplemented in Rust as part of the **Code Resurrection 2026** hackathon, with a focus on preserving behavior while adopting modern Rust engineering practices.

---

# License

This project follows the licensing terms of the original TinyExpr project. Please refer to the LICENSE file for details.
