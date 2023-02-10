mod common;

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
