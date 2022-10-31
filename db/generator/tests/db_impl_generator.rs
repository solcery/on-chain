use trybuild::TestCases;

#[test]
fn ui() {
    let t = TestCases::new();
    t.pass("tests/cases/01-correct_use.rs");
}
