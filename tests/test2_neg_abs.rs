mod common;

#[test]
fn test_safe_neg_abs() {
    common::test(
        "test_safe_neg_abs",
        "test_safe_neg_abs",
        "
            fn test_safe_neg_abs(mut x: i32) -> i32 {
                if x > 0 {
                    x = -1 * x;
                }
            
                assert!(x <= 0);
                x
            }        
        ",
        true,
    );
}