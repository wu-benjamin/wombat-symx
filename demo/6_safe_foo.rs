fn safe_foo(c1: bool, c2: bool, mut x: i32) -> i32 {
    if x > i32::MAX - 4 {
        x = i32::MAX - 4;
    }
    let mut r = if c1 { x + 3 } else { x + 4 };

    r = if c2 { r - 1 } else { r - 2 };
    assert!(r > x);
    r
}

fn main() {
    println!("{:p}", safe_foo as *const ());
}