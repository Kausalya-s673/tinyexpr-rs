//! Port of `example.c`: evaluate a single fixed expression and check the
//! result, the same "smallest possible usage" demo upstream ships.

mod common;
use common::*;

#[test]
fn sqrt_of_sum_of_squares() {
    let expr = "sqrt(5^2+7^2+11^2+(8-2)^2)";
    let value = interp(expr).unwrap_or_else(|e| panic!("failed to evaluate {expr:?}: {e}"));
    assert_close(value, 15.198684153570664);
}