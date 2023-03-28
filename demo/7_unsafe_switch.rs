// target/release/wombat_symx demo/7_unsafe_switch.rs unsafe_switch
fn unsafe_switch(x: i64) -> i64 {
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
        // Panics if x == i64::MIN
        _=>x-1
    }
}

fn main() {
    println!("{:p}", unsafe_switch as *const ());
}
