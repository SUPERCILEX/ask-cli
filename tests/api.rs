#[test]
fn fmt() {
    supercilex_tests::fmt();
}

#[test]
fn clippy() {
    supercilex_tests::clippy();
}

#[test]
fn api() {
    supercilex_tests::api();
}

#[test]
fn full() {
    trycmd::TestCases::new().case("tests/cmds/*.toml");
}
