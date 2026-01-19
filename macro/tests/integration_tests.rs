#[test]
fn test_macro() {
    let result = ppx_macros::include_ppx_string!("#param A\nA", ".", ["Hello world!"]);
    assert_eq!(result, "Hello world!");
}
