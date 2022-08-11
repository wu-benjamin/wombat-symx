fn assert_safe(x: i32) -> () {
    assert!(x == x);
}

fn assert_unsafe(x: i32) -> () {
    assert!(x < 13);
}

fn main() {
    assert_safe(12);
    assert_unsafe(-250);
}
