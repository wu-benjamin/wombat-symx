// target/release/wombat_symx demo/2_neg_abs.rs neg_abs
fn neg_abs(mut x: i16) -> i16 {
    if x > 0 {
        x = -1 * x;
    }

    assert!(x <= 0);
    x
}

fn main() {
    println!("{:p}", neg_abs as *const ());
}
