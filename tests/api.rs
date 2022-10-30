#[test]
fn readme() {
    trycmd::TestCases::new().case("tests/cmds/*.toml");
}
