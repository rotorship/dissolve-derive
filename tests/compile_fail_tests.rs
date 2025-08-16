use trybuild::TestCases;

#[test]
fn compile_fail_tests() {
	TestCases::new().compile_fail("tests/compile_fails/*.rs");
}
