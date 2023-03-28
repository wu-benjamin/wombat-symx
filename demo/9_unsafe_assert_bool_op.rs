// target/release/wombat_symx demo/9_unsafe_assert_bool_op.rs unsafe_assert_bool_op
fn unsafe_assert_bool_op(x: i32) -> () {
    assert!(x > 0 && x < 13);
}

fn main() {
    println!("{:p}", unsafe_assert_bool_op as *const ());
}
