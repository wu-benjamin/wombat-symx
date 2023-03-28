fn abs(mut x: i16) -> i16 {
    if x < 0 {
        // Panics if x == i16::MIN
        x = -1 * x;
    }

    assert!(x >= 0);
    x
}

fn main() {
    println!("{:p}", abs as *const ());
}