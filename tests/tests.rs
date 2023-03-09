// Copyright (c) 2023 Benjamin Jialong Wu
// This code is licensed under MIT license (see LICENSE.md for details)

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
fn test_unsafe_abs_i8() {
    common::test(
        "test_unsafe_abs_i8",
        "test_unsafe_abs_i8",
        "
            fn test_unsafe_abs_i8(mut x: i8) -> i8 {
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
fn test_unsafe_abs_i16() {
    common::test(
        "test_unsafe_abs_i16",
        "test_unsafe_abs_i16",
        "
            fn test_unsafe_abs_i16(mut x: i16) -> i16 {
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
fn test_unsafe_abs_i64() {
    common::test(
        "test_unsafe_abs_i64",
        "test_unsafe_abs_i64",
        "
            fn test_unsafe_abs_i64(mut x: i64) -> i64 {
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
fn test_safe_neg_abs_i8() {
    common::test(
        "test_safe_neg_abs_i8",
        "test_safe_neg_abs_i8",
        "
            fn test_safe_neg_abs_i8(mut x: i8) -> i8 {
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
fn test_safe_neg_abs_i16() {
    common::test(
        "test_safe_neg_abs_i16",
        "test_safe_neg_abs_i16",
        "
            fn test_safe_neg_abs_i16(mut x: i16) -> i16 {
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
fn test_safe_neg_abs_i64() {
    common::test(
        "test_safe_neg_abs_i64",
        "test_safe_neg_abs_i64",
        "
            fn test_safe_neg_abs_i64(mut x: i64) -> i64 {
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
fn test_safe_func_call() {
    common::test(
        "test_safe_func_call",
        "test_safe_func_call",
        "
            fn abs(x: i32) -> i32 {
                return x * -1;
            }
            fn test_safe_func_call() -> () {
                let y = abs(-13);
                assert!(y == 13);
            }        
        ",
        true,
    );
}

#[test]
fn test_unsafe_func_call() {
    common::test(
        "test_unsafe_func_call",
        "test_unsafe_func_call",
        "
            fn abs(x: i32) -> i32 {
                return x * -1;
            }
            fn test_unsafe_func_call() -> () {
                let y = abs(-2147483648);
                assert!(y >= 0);
            }        
        ",
        false,
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
fn test_safe_sequential_branch_1() {
    common::test(
        "test_safe_sequential_branch_1",
        "test_safe_sequential_branch_1",
        "
            fn test_safe_sequential_branch_1(
                c1: bool
            ) -> i32 {
                let r1 = if c1 {
                    1
                } else {
                    0
                };
                let r = r1;
                assert!(r >= 0);
                return r;
            }
        ",
        true,
    );
}

#[test]
fn test_safe_sequential_branch_2() {
    common::test(
        "test_safe_sequential_branch_2",
        "test_safe_sequential_branch_2",
        "
            fn test_safe_sequential_branch_2(
                c1: bool,
                c2: bool
            ) -> i32 {
                let r1 = if c1 {
                    1
                } else {
                    0
                };
                let r2 = if c2 {
                    2
                } else {
                    0
                };
                let r = r1 + r2;
                assert!(r >= 0);
                return r;
            } 
        ",
        true,
    );
}

#[test]
fn test_safe_sequential_branch_5() {
    common::test(
        "test_safe_sequential_branch_5",
        "test_safe_sequential_branch_5",
        "
            fn test_safe_sequential_branch_5(
                c1: bool,
                c2: bool,
                c3: bool,
                c4: bool,
                c5: bool
            ) -> i32 {
                let r1 = if c1 {
                    1
                } else {
                    0
                };
                let r2 = if c2 {
                    2
                } else {
                    0
                };
                let r3 = if c3 {
                    4
                } else {
                    0
                };
                let r4 = if c4 {
                    8
                } else {
                    0
                };
                let r5 = if c5 {
                    16
                } else {
                    0
                };
                let r = r1 + r2 + r3 + r4 + r5;
                assert!(r >= 0);
                return r;
            }
        ",
        true,
    );
}

#[test]
fn test_unsafe_sequential_branch_1() {
    common::test(
        "test_unsafe_sequential_branch_1",
        "test_unsafe_sequential_branch_1",
        "
            fn test_unsafe_sequential_branch_1(
                c1: bool
            ) -> i32 {
                let r1 = if c1 {
                    -1
                } else {
                    0
                };
                let r = r1;
                assert!(r >= 0);
                return r;
            }
        ",
        false,
    );
}

#[test]
fn test_unsafe_sequential_branch_2() {
    common::test(
        "test_unsafe_sequential_branch_2",
        "test_unsafe_sequential_branch_2",
        "
            fn test_unsafe_sequential_branch_2(
                c1: bool,
                c2: bool
            ) -> i32 {
                let r1 = if c1 {
                    -1
                } else {
                    0
                };
                let r2 = if c2 {
                    2
                } else {
                    0
                };
                let r = r1 + r2;
                assert!(r >= 0);
                return r;
            } 
        ",
        false,
    );
}

#[test]
fn test_unsafe_sequential_branch_5() {
    common::test(
        "test_unsafe_sequential_branch_5",
        "test_unsafe_sequential_branch_5",
        "
            fn test_unsafe_sequential_branch_5(
                c1: bool,
                c2: bool,
                c3: bool,
                c4: bool,
                c5: bool
            ) -> i32 {
                let r1 = if c1 {
                    -1
                } else {
                    0
                };
                let r2 = if c2 {
                    2
                } else {
                    0
                };
                let r3 = if c3 {
                    -4
                } else {
                    0
                };
                let r4 = if c4 {
                    8
                } else {
                    0
                };
                let r5 = if c5 {
                    -16
                } else {
                    0
                };
                let r = r1 + r2 + r3 + r4 + r5;
                assert!(r >= 0);
                return r;
            }
        ",
        false,
    );
}

#[test]
fn test_unsafe_tricky_phi() {
    common::test(
        "test_unsafe_tricky_phi",
        "test_unsafe_tricky_phi",
        "
            fn test_unsafe_tricky_phi(
                c1: bool,
                c2: bool
            ) -> i32 {
                let mut r = 0;
                if c1 {
                    r -= 1;
                }
                if c2 {
                    r += 2;
                }
                assert!(r >= 0);
                return r;
            }
        ",
        false,
    );
}

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
