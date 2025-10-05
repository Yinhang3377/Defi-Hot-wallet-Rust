#[allow(unused_attributes)]
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(any(miri, not(feature = "std")), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
