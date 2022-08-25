fn neg_abs(mut x_testme: i32, mut y_testme: i32, easyfindme: i32) -> i32 {
    if x_testme > 0 {
        x_testme = -1 * x_testme
    }

    if y_testme > 0 {
        y_testme = -1 * y_testme
    }

    assert!(x_testme <= 0 && y_testme <= 0);
    x_testme + easyfindme + y_testme
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
    neg_abs(5, 3, 4);
    abs(5);
}
