fn neg_abs(mut x_mut: i32) -> i32 {
    if x_mut > 0 {
        x_mut = -1 * x_mut
    }

    assert!(x_mut <= 0);
    x_mut - 1
}

fn func_names(mut x_mut: i32, mut y_mut: i32, z_non_mut: i32) -> i32 {
    // x_mut must be non-positive
    if x_mut > 0 {
        x_mut = -1 * x_mut
    }

    // y_mut must be non-positive
    if y_mut > 0 {
        y_mut = -1 * y_mut
    }

    // x_mut & y_mut both non-positive
    assert!(x_mut <= 0);
    assert!(y_mut <= 0);

    x_mut + z_non_mut + y_mut
}

fn abs(mut x: i32) -> i32 {
    if x < 0 {
        x = -1 * x;
    }

    assert!(x >= 0);
    x
}

fn main() {
    println!("Hello, world!");
    neg_abs(5);
    func_names(5, 2, 3);
    abs(5);
}
