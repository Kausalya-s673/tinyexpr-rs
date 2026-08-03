//! Port of `smoke.c`, upstream TinyExpr's main test suite.
//!
//! Each C `test_*` function (registered via `lrun("Name", test_*)` in
//! `smoke.c`'s `main`) becomes one `#[test] fn` here, keeping the same
//! grouping so it's easy to cross-reference against the original.
//!
//! Three categories of cases from the original suite could **not** be
//! ported as-is, because they exercise behavior this Rust port doesn't
//! (yet, or by design) replicate bit-for-bit:
//!
//! 1. **No parse-error position codes.** Upstream's `te_interp`/
//!    `te_compile` report a character offset (`int *error`) on failure.
//!    This port's `ParseError` is a plain enum with no position. Ported
//!    syntax-error cases assert only that the expression fails overall
//!    (see `syntax()` below).
//! 2. **No scientific notation in the lexer.** `lexer.rs` only consumes
//!    ASCII digits and `.` for a number token; `1e3` lexes as `Number(1)`
//!    followed by a separate `Identifier("e3")` token, not `1000.0`. Test
//!    cases from `smoke.c` that relied on `1e3`/`5e-5`-style literals are
//!    rewritten below using plain decimal literals with an equivalent
//!    value, and `scientific_notation_is_not_supported` documents the gap
//!    directly.
//! 3. **No custom user functions or closures.** `test_dynamic` and
//!    `test_closure` in `smoke.c` register C callbacks (`TE_FUNCTION2`,
//!    `TE_CLOSURE1`, ...) as variables. The `Builtin` enum here is a
//!    fixed, closed set with no such registration mechanism (this
//!    matches the "support custom user functions" / "support closures"
//!    TODOs on tinyexpr-rs). Both are kept below as `#[ignore]`d stubs
//!    that explain the gap rather than silently dropping the coverage.
//!
//! A fourth, more surprising divergence shows up in `infs()`: this port's
//! `fac`/`ncr`/`npr` accumulate in plain `f64` instead of upstream's
//! 64-bit-integer-with-overflow-guard, so it can exactly represent
//! several values upstream artificially overflows to `INFINITY` well
//! before double's real ~1.8e308 ceiling. See that test for specifics.

mod common;
use common::*;
use tinyexpr_rs::errors::ParseError;

// ---------------------------------------------------------------------
// test_results
// ---------------------------------------------------------------------

#[test]
fn results() {
    let cases: &[(&str, f64)] = &[
        ("1", 1.0),
        ("1 ", 1.0),
        ("(1)", 1.0),
        ("pi", 3.14159),
        ("atan(1)*4 - pi", 0.0),
        ("e", 2.71828),
        ("2+1", 2.0 + 1.0),
        ("(((2+(1))))", 2.0 + 1.0),
        ("3+2", 3.0 + 2.0),
        ("3+2+4", 3.0 + 2.0 + 4.0),
        ("(3+2)+4", 3.0 + 2.0 + 4.0),
        ("3+(2+4)", 3.0 + 2.0 + 4.0),
        ("(3+2+4)", 3.0 + 2.0 + 4.0),
        ("3*2*4", 3.0 * 2.0 * 4.0),
        ("(3*2)*4", 3.0 * 2.0 * 4.0),
        ("3*(2*4)", 3.0 * 2.0 * 4.0),
        ("(3*2*4)", 3.0 * 2.0 * 4.0),
        ("3-2-4", 3.0 - 2.0 - 4.0),
        ("(3-2)-4", (3.0 - 2.0) - 4.0),
        ("3-(2-4)", 3.0 - (2.0 - 4.0)),
        ("(3-2-4)", 3.0 - 2.0 - 4.0),
        ("3/2/4", 3.0 / 2.0 / 4.0),
        ("(3/2)/4", (3.0 / 2.0) / 4.0),
        ("3/(2/4)", 3.0 / (2.0 / 4.0)),
        ("(3/2/4)", 3.0 / 2.0 / 4.0),
        ("(3*2/4)", 3.0 * 2.0 / 4.0),
        ("(3/2*4)", 3.0 / 2.0 * 4.0),
        ("3*(2/4)", 3.0 * (2.0 / 4.0)),
        ("asin sin .5", 0.5),
        ("sin asin .5", 0.5),
        ("ln exp .5", 0.5),
        ("exp ln .5", 0.5),
        ("asin sin-.5", -0.5),
        ("asin sin-0.5", -0.5),
        ("asin sin -0.5", -0.5),
        ("asin (sin -0.5)", -0.5),
        ("asin (sin (-0.5))", -0.5),
        ("asin sin (-0.5)", -0.5),
        ("(asin sin (-0.5))", -0.5),
        ("log10 1000", 3.0),
        ("log10(1000)", 3.0),
        // `log 1000` == 3, not 6.9078: this port always treats `log` as
        // base-10 (matching upstream's *default* build, i.e. without the
        // `TE_NAT_LOG` compile flag) — see the comment on `Builtin::Log`
        // in eval.rs. `ln` is the natural log.
        ("log 1000", 3.0),
        ("ln (e^10)", 10.0),
        ("100^.5+1", 11.0),
        ("100 ^.5+1", 11.0),
        ("100^+.5+1", 11.0),
        ("100^--.5+1", 11.0),
        ("100^---+-++---++-+-+-.5+1", 11.0),
        ("100^-.5+1", 1.1),
        ("100^---.5+1", 1.1),
        ("100^+---.5+1", 1.1),
        // Originally `1e2^+---.5e0+1e0` / `--(1e2^(+(-(-(-.5e0))))+1e0)` —
        // rewritten with plain decimals since `1e2`/`.5e0`/`1e0` don't
        // lex as numbers here (see module docs, point 2).
        ("100^+---.5+1.0", 1.1),
        ("--(100^(+(-(-(-.5))))+1.0)", 1.1),
        ("sqrt 100 + 7", 17.0),
        ("sqrt 100 * 7", 70.0),
        ("sqrt (100 * 100)", 100.0),
        ("1,2", 2.0),
        ("1,2+1", 3.0),
        ("1+1,2+2,2+1", 3.0),
        ("1,2,3", 3.0),
        ("(1,2),3", 3.0),
        ("1,(2,3)", 3.0),
        ("-(1,(2,3))", -3.0),
        ("2^2", 4.0),
        ("pow(2,2)", 4.0),
        ("atan2(1,1)", 0.7854),
        ("atan2(1,2)", 0.4636),
        ("atan2(2,1)", 1.1071),
        ("atan2(3,4)", 0.6435),
        ("atan2(3+3,4*2)", 0.6435),
        ("atan2(3+3,(4*2))", 0.6435),
        ("atan2((3+3),4*2)", 0.6435),
        ("atan2((3+3),(4*2))", 0.6435),
    ];

    for &(expr, answer) in cases {
        match interp(expr) {
            Ok(value) => assert_close(value, answer),
            Err(e) => panic!("FAILED: {expr:?} ({e})"),
        }
    }
}

/// Documents a real gap versus upstream: `lexer.rs` has no exponent
/// notation, so `1e3` never becomes the number `1000.0`. It lexes as
/// `Number(1.0)` immediately followed by an `Identifier("e3")` token,
/// which is always a trailing/unexpected token once the (one-token)
/// number production has already returned — so the overall parse fails.
#[test]
fn scientific_notation_is_not_supported() {
    assert!(interp("1e3").is_err());
    assert!(interp("5e-5").is_err());
    assert!(interp("1.0e3").is_err());
}

// ---------------------------------------------------------------------
// test_syntax
// ---------------------------------------------------------------------

/// Upstream's `test_syntax` checks both the *value* (`te_interp` returns
/// `NaN`) and the exact *error position* (`te_compile`'s `int *error`)
/// for a list of malformed expressions. This port's `ParseError` has no
/// position information, so each case here only asserts that the
/// expression fails overall — either at parse time, or (for cases that
/// parse successfully but reference an unbound identifier, since this
/// port checks that at `eval` time rather than upstream's compile time)
/// at eval time against an empty variable map.
#[test]
fn syntax() {
    let bad_exprs = [
        "", "1+", "1)", "(1", "1**1", "1*2(+4", "1*2(1+4", "a+5", "!+5", "_a+5", "#a+5", "1^^5",
        "1**5", "sin(cos5",
    ];

    for expr in bad_exprs {
        let result = interp(expr);
        assert!(
            result.is_err(),
            "expected {expr:?} to fail (parse or eval), got {result:?}"
        );
    }
}

// ---------------------------------------------------------------------
// test_nans
// ---------------------------------------------------------------------

#[test]
fn nans() {
    let nan_exprs = [
        "0/0",
        "1%0",
        "1%(1%0)",
        "(1%0)%1",
        "fac(-1)",
        "ncr(2, 4)",
        "ncr(-2, 4)",
        "ncr(2, -4)",
        "npr(2, 4)",
        "npr(-2, 4)",
        "npr(2, -4)",
    ];

    for expr in nan_exprs {
        let value = interp(expr).unwrap_or_else(|e| panic!("{expr:?} should parse+eval: {e}"));
        assert!(value.is_nan(), "{expr:?} => {value}, expected NaN");
    }
}

// ---------------------------------------------------------------------
// test_infs
// ---------------------------------------------------------------------

/// Upstream's `test_infs` expects every one of these to evaluate to
/// `±INFINITY`. In the C original, `fac`/`ncr`/`npr` compute with 64-bit
/// *integer* arithmetic and deliberately return `INFINITY` the moment
/// that would overflow (`ULONG_MAX`, ~1.8e19) — long before the
/// mathematical result actually exceeds what an `f64` can hold
/// (~1.8e308).
///
/// This port's `factorial`/`ncr`/`npr` (see `eval.rs`) accumulate
/// directly in `f64` with no such integer-overflow guard. That means
/// several of upstream's "overflow" cases are, in this port, genuinely
/// representable finite doubles — they only diverge from upstream in
/// *when* they overflow, not in being wrong. Concretely:
///
/// - `ncr(300,100)` ≈ 4.16e81 — comfortably finite in `f64`.
/// - `npr(100,90)` ≈ 2.57e151 — likewise finite.
/// - `npr(30,25)` ≈ 2.21e30 — likewise finite.
///
/// while the truly astronomical cases (`fac(300)` ≈ 3.06e614,
/// `ncr(300000,100)` ≈ 1e390) still overflow `f64` itself and correctly
/// come out as `INFINITY` in both implementations. Both groups are
/// checked explicitly below rather than asserting `INFINITY` uniformly.
#[test]
fn infs() {
    let inf_exprs = [
        "1/0",
        "log(0)",
        "pow(2,10000000)",
        "fac(300)",
        "ncr(300000,100)",
        "ncr(300000,100)*8",
        "npr(3,2)*ncr(300000,100)",
    ];
    for expr in inf_exprs {
        let value = interp(expr).unwrap_or_else(|e| panic!("{expr:?} should parse+eval: {e}"));
        assert!(
            value.is_infinite(),
            "{expr:?} => {value}, expected +/-infinity"
        );
    }
}

#[test]
fn infs_upstream_diverges_still_finite_here() {
    // See `infs()`'s doc comment: these are the three upstream
    // "overflow" cases that this f64-based port can represent exactly.
    let finite_but_huge: &[(&str, f64)] = &[
        ("ncr(300,100)", 4.158251463258559e81),
        ("npr(100,90)", 2.5718203109552512e151),
        ("npr(30,25)", 2.210440498434926e30),
    ];
    for &(expr, expected) in finite_but_huge {
        let value = interp(expr).unwrap_or_else(|e| panic!("{expr:?} should parse+eval: {e}"));
        assert!(value.is_finite(), "{expr:?} => {value}, expected finite");
        let rel_diff = (value - expected).abs() / expected;
        assert!(
            rel_diff < 1e-9,
            "{expr:?} => {value}, expected ~{expected} (rel diff {rel_diff})"
        );
    }
}

// ---------------------------------------------------------------------
// test_variables
// ---------------------------------------------------------------------

#[test]
fn variables() {
    for i in 0..5 {
        let x = i as f64;
        let y = 2.0; // upstream's outer loop is `for (y = 2; y < 3; ++y)` — a single value.
        let v = vars([("x", x), ("y", y), ("te_st", x)]);

        assert_close(interp_with("cos x + sin y", &v).unwrap(), x.cos() + y.sin());
        assert_close(interp_with("x+x+x-y", &v).unwrap(), x + x + x - y);
        assert_close(interp_with("x*y^3", &v).unwrap(), x * y * y * y);
        assert_close(interp_with("te_st+5", &v).unwrap(), x + 5.0);
    }
}

/// Upstream compiles `expr5`..`expr8` against a *fixed* variable list
/// (`{"x", ...}, {"y", ...}, {"te_st", ...}`) and expects `te_compile` to
/// fail outright for each, because the identifier used isn't in that
/// list. This port has no such compile-time check — `parser::parse`
/// never looks at a variable list at all, so an unrecognized identifier
/// always parses fine as `Expr::Variable` and only fails once `eval` is
/// asked to resolve it against a map that doesn't contain it.
///
/// Two of the four cases below (`sinn x`, `si x`) still fail at *parse*
/// time here, but for an unrelated reason: `sinn`/`si` aren't builtins,
/// so they parse as a lone `Variable`, leaving the following `x` as a
/// trailing token the grammar doesn't accept — not because the compiler
/// checked and rejected the name.
#[test]
fn variables_unknown_identifiers_still_fail_overall() {
    let v = vars([("x", 1.0), ("y", 2.0), ("te_st", 3.0)]);

    // Parses fine (xx is just an unbound Variable); fails at eval time.
    assert!(matches!(
        interp_with("xx*y^3", &v),
        Err(ParseError::UnknownIdentifier(name)) if name == "xx"
    ));

    // Parses fine (tes is just an unbound Variable); fails at eval time.
    assert!(matches!(
        interp_with("tes", &v),
        Err(ParseError::UnknownIdentifier(name)) if name == "tes"
    ));

    // Fails to parse: "sinn" isn't a builtin, so it's a lone Variable,
    // and the following "x" is then an unexpected trailing token.
    assert!(interp_with("sinn x", &v).is_err());

    // Same story: "si" isn't a builtin either.
    assert!(interp_with("si x", &v).is_err());
}

// ---------------------------------------------------------------------
// test_functions
// ---------------------------------------------------------------------

/// Cross-checks every builtin against Rust's own `f64` methods. Since
/// `eval.rs`'s `eval_function` is itself implemented directly in terms
/// of these same methods, this is intentionally closer to a "does the
/// parser dispatch to the right builtin" check than an independent
/// numerical verification — which is exactly the role `test_functions`
/// plays in upstream too (it cross-checks against the C standard
/// library's `libm`, the same library TinyExpr's C implementation calls
/// into).
///
/// Where the "trusted" side of a comparison is itself `NaN` (e.g.
/// `acos(x)` for `x` outside `[-1, 1]`), the case is skipped rather than
/// asserted — mirroring upstream's `cross_check` macro, which does
/// `if ((b)!=(b)) break;` before comparing.
#[test]
fn functions() {
    fn cross_check(expr: &str, v: &std::collections::HashMap<String, f64>, expected: f64) {
        if expected.is_nan() {
            return;
        }
        let got = interp_with(expr, v).unwrap_or_else(|e| panic!("{expr:?} failed: {e}"));
        assert_close(got, expected);
    }

    let mut x = -5.0_f64;
    while x < 5.0 {
        let v = vars([("x", x), ("y", 0.0)]);

        cross_check("abs x", &v, x.abs());
        cross_check("acos x", &v, x.acos());
        cross_check("asin x", &v, x.asin());
        cross_check("atan x", &v, x.atan());
        cross_check("ceil x", &v, x.ceil());
        cross_check("cos x", &v, x.cos());
        cross_check("cosh x", &v, x.cosh());
        cross_check("exp x", &v, x.exp());
        cross_check("floor x", &v, x.floor());
        cross_check("ln x", &v, x.ln());
        cross_check("log10 x", &v, x.log10());
        cross_check("sin x", &v, x.sin());
        cross_check("sinh x", &v, x.sinh());
        cross_check("sqrt x", &v, x.sqrt());
        cross_check("tan x", &v, x.tan());
        cross_check("tanh x", &v, x.tanh());

        if x.abs() >= 0.01 {
            let mut y = -2.0_f64;
            while y < 2.0 {
                let v2 = vars([("x", x), ("y", y)]);
                cross_check("atan2(x,y)", &v2, x.atan2(y));
                cross_check("pow(x,y)", &v2, x.powf(y));
                y += 0.2;
            }
        }

        x += 0.2;
    }
}

// ---------------------------------------------------------------------
// test_dynamic / test_closure — NOT PORTABLE, see module docs point 3.
// ---------------------------------------------------------------------

#[test]
#[ignore = "no custom-function registration in this port yet: `Builtin` is a \
            fixed, closed enum (abs/acos/.../tanh) with no way to bind a \
            user-supplied fn (upstream's TE_FUNCTION0..7) as a callable \
            identifier. This mirrors the 'support custom user functions' \
            TODO on tinyexpr-rs. Un-ignore once that lands, replacing the \
            `te_variable {\"sum2\", sum2, TE_FUNCTION2}`-style bindings from \
            smoke.c's test_dynamic with whatever this crate's equivalent \
            registration API turns out to be."]
fn dynamic_user_functions_unsupported() {
    unimplemented!("custom user functions are not yet supported by this port");
}

#[test]
#[ignore = "no closures (upstream's TE_CLOSURE0..2 with a `void *context`) \
            in this port either, for the same reason as \
            dynamic_user_functions_unsupported above. smoke.c's \
            test_closure exercises a context pointer threaded through the \
            callback (clo0/clo1/clo2/cell); there's currently no Rust-side \
            equivalent to bind."]
fn closures_unsupported() {
    unimplemented!("closures are not yet supported by this port");
}

// ---------------------------------------------------------------------
// test_optimize
// ---------------------------------------------------------------------

/// Upstream checks `ex->value` directly on the *compiled* (but not yet
/// evaluated) expression to confirm constant folding already computed
/// the answer. The equivalent here is `Expr::optimize()`: for a fully
/// constant expression it should collapse straight to `Expr::Number`,
/// with that number already equal to the final answer — no `eval` call
/// needed to discover it.
#[test]
fn optimize() {
    let cases: &[(&str, f64)] = &[
        ("5+5", 10.0),
        ("pow(2,2)", 4.0),
        ("sqrt 100", 10.0),
        ("pi * 2", 6.2832),
    ];

    for &(expr, answer) in cases {
        let optimized = parse_ok(expr).optimize();
        match optimized {
            tinyexpr_rs::ast::Expr::Number(value)=> assert_close(value, answer),
            other => panic!("{expr:?} should fold to a constant, got {other:?}"),
        }
        // Still agrees with a full (re-parsed, non-optimized) eval.
        assert_close(interp(expr).unwrap(), answer);
    }
}

// ---------------------------------------------------------------------
// test_pow
// ---------------------------------------------------------------------

/// Upstream's `test_pow` is `#ifdef`-gated on `TE_POW_FROM_RIGHT` and
/// picks one of two equivalence tables depending on which associativity
/// the build was compiled with. This port only implements the *default*
/// (non-`TE_POW_FROM_RIGHT`) behavior — `^` is left-associative,
/// spreadsheet-style (`parse_factor`'s doc comment in parser.rs is
/// explicit about this) — so only that table is ported.
#[test]
fn pow_associativity() {
    let v = vars([("a", 2.0), ("b", 3.0)]);

    let equivalences: &[(&str, &str)] = &[
        ("2^3^4", "(2^3)^4"),
        ("-2^2", "(-2)^2"),
        ("(-2)^2", "4"),
        ("--2^2", "2^2"),
        ("---2^2", "(-2)^2"),
        ("-2^2", "4"),
        ("2^1.1^1.2^1.3", "((2^1.1)^1.2)^1.3"),
        ("-a^b", "(-a)^b"),
        ("-a^-b", "(-a)^(-b)"),
        ("1^0", "1"),
        ("(1)^0", "1"),
        ("(-1)^0", "1"),
        ("(-5)^0", "1"),
        ("-2^-3^-4", "((-2)^(-3))^(-4)"),
    ];

    for &(lhs, rhs) in equivalences {
        let r1 = interp_with(lhs, &v).unwrap_or_else(|e| panic!("{lhs:?} failed: {e}"));
        let r2 = interp_with(rhs, &v).unwrap_or_else(|e| panic!("{rhs:?} failed: {e}"));
        assert!(
            (r1 - r2).abs() <= 1e-9,
            "{lhs:?} ({r1}) != {rhs:?} ({r2})"
        );
    }
}

// ---------------------------------------------------------------------
// test_combinatorics
// ---------------------------------------------------------------------

/// Two cases from upstream's table, `fac(0.2)` and `fac(4.8)`, are
/// deliberately **not** ported with their original expected values (both
/// `1` and `24` respectively). Upstream's C `fac()` casts its `double`
/// argument to an integer type, silently truncating (`(unsigned)(4.8)`
/// == `4`, so `fac(4.8)` == `4! == 24`). This port's `factorial` (see
/// `eval.rs`) instead treats any non-integer input as a domain error and
/// returns `NaN` — arguably the more defensible behavior (it doesn't
/// quietly reinterpret `4.8` as `4`), but a genuine, intentional
/// divergence from upstream. See `combinatorics_fractional_input`.
#[test]
fn combinatorics() {
    let cases: &[(&str, f64)] = &[
        ("fac(0)", 1.0),
        ("fac(1)", 1.0),
        ("fac(2)", 2.0),
        ("fac(3)", 6.0),
        ("fac(10)", 3628800.0),
        ("ncr(0,0)", 1.0),
        ("ncr(10,1)", 10.0),
        ("ncr(10,0)", 1.0),
        ("ncr(10,10)", 1.0),
        ("ncr(16,7)", 11440.0),
        ("ncr(16,9)", 11440.0),
        ("ncr(100,95)", 75287520.0),
        ("npr(0,0)", 1.0),
        ("npr(10,1)", 10.0),
        ("npr(10,0)", 1.0),
        ("npr(10,10)", 3628800.0),
        ("npr(20,5)", 1860480.0),
        ("npr(100,4)", 94109400.0),
    ];

    for &(expr, answer) in cases {
        match interp(expr) {
            Ok(value) => assert_close(value, answer),
            Err(e) => panic!("FAILED: {expr:?} ({e})"),
        }
    }
}

#[test]
fn combinatorics_fractional_input() {
    // See combinatorics()'s doc comment: this port requires an exact
    // integer, unlike upstream's truncating cast.
    assert!(interp("fac(0.2)").unwrap().is_nan());
    assert!(interp("fac(4.8)").unwrap().is_nan());
}