fn test_safe(x: i32) -> i32 {
    match x{
        1=>{
            i32::MAX - 4
        },  
        2=>{
            x+1
        },
        2147483647=>{
            x
        }
        _=>x+1
    }
}

fn test_unsafe(x: i64) -> i64 {
    if x > 1 {
        let y = 2;
        y
    } else {
        match x{
            1=>{
                x
            },  
            2=>{
                x
            },
            3=>{
                x-1
            },
            _=>x+1
        }
    }
}


fn main() {
    test_safe(5);
    test_unsafe(5);
}
