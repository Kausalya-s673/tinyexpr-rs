const TEST_CASES: &[(&str, f64)] = &[
    ("2+2", 4.0),
    ("2*3+4", 10.0),
    ("2*(3+4)", 14.0),
    ("2^3", 8.0),
    ("--5", 5.0),
    ("sin(0)", 0.0),
    ("pow(2,5)", 32.0),
    ("sqrt(16)", 4.0),
];

#[test]
fn test_cases_are_defined() {
    assert_eq!(TEST_CASES.len(), 8);

    assert_eq!(TEST_CASES[0], ("2+2", 4.0));
    assert_eq!(TEST_CASES[1], ("2*3+4", 10.0));
    assert_eq!(TEST_CASES[2], ("2*(3+4)", 14.0));
    assert_eq!(TEST_CASES[3], ("2^3", 8.0));
    assert_eq!(TEST_CASES[4], ("--5", 5.0));
    assert_eq!(TEST_CASES[5], ("sin(0)", 0.0));
    assert_eq!(TEST_CASES[6], ("pow(2,5)", 32.0));
    assert_eq!(TEST_CASES[7], ("sqrt(16)", 4.0));
}