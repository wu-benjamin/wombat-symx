fn abs(mut x: i32) -> i32 {
    if x < 0 {
        x = -1 * x;
    }

    assert!(x >= 0);
    x
}

fn main() {
    abs(5);
}