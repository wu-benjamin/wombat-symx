mod common;

// spec foo {
//     ensures r > x;
// }

#[test]
fn test_unsafe_foo() {
    common::test(
        "test_unsafe_foo",
        "test_unsafe_foo",
        "
            fn test_unsafe_foo(c1: bool, c2: bool, x: i32) -> i32 {
                let mut r = if c1 { x + 3 } else { x + 4 };
            
                r = if c2 { r - 1 } else { r - 2 };
                assert!(r > x);
                r
            }
        ",
        false,
    );
}

#[test]
fn test_safe_foo() {
    common::test(
        "test_safe_foo",
        "test_safe_foo",
        "
            fn test_safe_foo(c1: bool, c2: bool, mut x: i32) -> i32 {
                if x > i32::MAX - 4 {
                    x = i32::MAX - 4;
                }
                let mut r = if c1 { x + 3 } else { x + 4 };
            
                r = if c2 { r - 1 } else { r - 2 };
                assert!(r > x);
                r
            }
        ",
        true,
    );
}