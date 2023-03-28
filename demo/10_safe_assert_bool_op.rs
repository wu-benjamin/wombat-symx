// target/release/wombat_symx demo/10_safe_assert_bool_op.rs safe_assert_bool_op
fn safe_assert_bool_op(mut x: i32) -> () {
    if x > 0 {
        x -= 1;
    }
    assert!(x < 0 || x + 1 > 0);
}   

fn main() {
    println!("{:p}", safe_assert_bool_op as *const ());
}
