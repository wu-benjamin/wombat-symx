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


#[test]
fn test_safe_switch() {
    common::test(
        "test_safe_switch",
        "test_safe_switch",
        "
            fn test_safe_switch(x: i32) -> i32 {
                match x{
                    1=>{
                        i32::MAX - 4
                    },  
                    2=>{
                        x+3
                    },
                    2147483647=>{
                        x
                    }
                    _=>x+1
                }
            }
        ",
        true,
    );
}

#[test]
fn test_unsafe_switch() {
    common::test(
        "test_unsafe_switch",
        "test_unsafe_switch",
        "
            fn test_unsafe_switch(x: i64) -> i64 {
                match x{
                    1=>{
                        x
                    },  
                    2=>{
                        x+4
                    },
                    3=>{
                        x+1
                    },
                    _=>x-1
                }
            }
        ",
        false,
    );
}

#[test]
#[ignore]
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

#[test]
#[ignore]
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
