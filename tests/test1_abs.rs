mod common;

#[test]
fn test_unsafe_abs() {
    common::test(
        "test_unsafe_abs",
        "test_unsafe_abs",
        "
            fn test_unsafe_abs(mut x: i32) -> i32 {
                if x < 0 {
                    x = -1 * x;
                }
            
                assert!(x >= 0);
                x
            }
        ",
        false,
    );
}