//! Port of `example3.c`: calling a user-supplied C function (`my_sum`,
//! registered as `TE_FUNCTION2`) from within an expression.
//!
//! This is **not portable** to the current API: `Builtin` (ast.rs) is a
//! fixed, closed enum of the built-in math functions, with no mechanism
//! to register an arbitrary Rust `fn`/closure under a name and call it
//! from a parsed expression the way `example3.c`'s `mysum(5, 6)` does.
//! This is the same gap flagged in `smoke.rs` for `test_dynamic`/
//! `test_closure` — see that file's module docs for more detail.
//!
//! Kept as an explicit `#[ignore]`d stub (rather than omitted entirely)
//! so the missing coverage is visible in `cargo test` output instead of
//! silently disappearing.

#[test]
#[ignore = "no custom user-function registration in this port yet — see \
            smoke.rs's `dynamic_user_functions_unsupported` for the full \
            explanation. Un-ignore and implement once the crate exposes a \
            way to bind a Rust fn (e.g. `fn my_sum(a: f64, b: f64) -> f64`) \
            to a callable identifier, then evaluate `mysum(5, 6)` == 11.0 \
            through it, matching example3.c."]
fn calling_a_custom_function_from_an_expression() {
    unimplemented!("custom user functions (upstream's TE_FUNCTION2, etc.) are not yet supported");
}
