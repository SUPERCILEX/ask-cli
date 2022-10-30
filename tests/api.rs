#[test]
fn full() {
    trycmd::TestCases::new().case("tests/cmds/*.toml");
}
