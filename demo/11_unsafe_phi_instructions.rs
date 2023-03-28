// target/release/wombat_symx demo/11_unsafe_phi_instructions.rs unsafe_complex_phi
fn unsafe_complex_phi(x: i8, y: i8, z: i8) -> i8 {
    // c1 is non-positive
    let mut c1 = x;
    if x > 0 {
        c1 = -1 * x;
    };

    // c2 is in range (0, 10) if y is in (-10, 0), else 11
    let mut c2 = 11;
    if y < 0 && y > -10 {
        c2 = -1 * y;
    };

    // c3 is -2 or 0
    let mut c3 = 0;
    if z == 10 {
        c3 = -2;
    };

    // This will panic if x = -128, y = -1, z = 10
    c1 + c2 + c3
} 

fn main() {
    println!("{:p}", unsafe_complex_phi as *const ());
}
