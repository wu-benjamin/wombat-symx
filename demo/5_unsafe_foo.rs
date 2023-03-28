// target/release/wombat_symx demo/5_unsafe_foo.rs unsafe_foo
fn unsafe_foo(c1: bool, c2: bool, x: i32) -> i32 {
    // Panics if x is close to i32::MAX
    let mut r = if c1 { x + 3 } else { x + 4 };

    r = if c2 { r - 1 } else { r - 2 };
    assert!(r > x);
    r
}

fn main() {
    println!("{:p}", unsafe_foo as *const ());
}
