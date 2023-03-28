// target/release/wombat_symx demo/4_safe_function_call.rs safe_func_call
fn abs(x: i32) -> i32 {
    if x < 0 {
        return x * -1;
    } else {
        return x;
    }
}

fn safe_func_call() -> () {
    let y = abs(-13);
    assert!(y == 13);
}   

fn main() {
    println!("{:p}", safe_func_call as *const ());
}
