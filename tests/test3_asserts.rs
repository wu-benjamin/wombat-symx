mod common;

#[test]
fn test_safe_assert() {
    common::test(
        "test_safe_assert",
        "test_safe_assert",
        "
            fn test_safe_assert(mut x: i32) -> () {
                if x > 0 {
                    x -= 1;
                }
                assert!(x < 0 || x + 1 > 0);
            }   
        ",
        true,
    );
}

#[test]
fn test_unsafe_assert() {
    common::test(
        "test_unsafe_assert",
        "test_unsafe_assert",
        "
            fn test_unsafe_assert(x: i32) -> () {
                assert!(x > 0 && x < 13);
            }
        ",
        false,
    );
}