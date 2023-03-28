// target/release/wombat_symx demo/3_unsafe_function_call.rs unsafe_func_call
fn abs(x: i32) -> i32 {
    if x < 0 {
        return x * -1;
    } else {
        return x;
    }
}

fn unsafe_func_call() -> () {
    let y = abs(-2147483648);
    assert!(y >= 0);
}   

fn main() {
    println!("{:p}", unsafe_func_call as *const ());
}
