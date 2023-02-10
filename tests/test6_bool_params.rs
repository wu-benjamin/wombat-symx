mod common;

#[test]
fn test_safe_bool_params() {
    common::test(
        "test_safe_bool_params",
        "test_safe_bool_params",
        "
            fn test_safe_bool_params(x: bool) -> () {
                assert!(x || !x);
            }
        ",
        true,
    );
}