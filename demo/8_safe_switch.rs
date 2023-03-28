fn safe_switch(x: i32) -> i32 {
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

fn main() {
    println!("{:p}", safe_switch as *const ());
}