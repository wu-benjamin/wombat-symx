// fn neg_abs(mut x: i32) -> i32 {
//     if x > 0 {
//         x = -1 * x;
//     }

//     assert!(x <= 0);
//     x
// }

fn abs(mut x: i32) -> i32 {
    if x < 0 {
        x = -1 * x;
    }

    assert!(x >= 0);
    x
}

fn main() {
    println!("Hello, world!");
    // neg_abs(5);
    abs(5);
}
