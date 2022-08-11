fn unsafe_foo(c1: bool, c2: bool, x: i32) -> i32 {
    let mut r = if c1 { x + 3 } else { x + 4 };

    r = if c2 { r - 1 } else { r - 2 };
    assert!(r > x);
    r
}

// spec foo {
//     ensures r > x;
// }

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
    unsafe_foo(false, true, 13);
    safe_foo(false, true, 13);
}
