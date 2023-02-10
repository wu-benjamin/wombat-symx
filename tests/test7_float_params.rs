mod common;

#[test]
fn test_unsafe_float_params() {
    common::test(
        "test_unsafe_float_params",
        "test_unsafe_float_params",
        "
            fn test_unsafe_float_params(y: f32) -> () {
                assert!(y == 0.0);
            }
        ",
        false,
    );
}