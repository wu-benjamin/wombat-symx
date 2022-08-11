fn test1() -> i32 {
    let mut x = 2;
    x = 7;
    assert!(x == 2);
    x
}

fn test2(mut x: i32) -> i32 {
    x = 2;
    let y = if x > 0 { -1 * x } else { x };
    assert!(y <= 0);
    return y;
}

fn test3() -> i32 {
    let mut x = 0;
    let mut sum = 0;
    while x < 10 {
        x = x + 1;
        sum = sum + x;
    }
    return sum;
}

fn minus_one_safe(x: i32) -> i32 {
    if x > 0 {
        return x - 1;
    }
    return x;
}

fn minus_one_unsafe(x: i32) -> i32 {
    return x - 1;
}

fn neg_abs(mut x: i32) -> i32 {
    if x > 0 {
        x = -1 * x;
    }
    return x;
}

fn abs(mut x: i32) -> i32 {
    if x < 0 {
        x = -1 * x;
    }
    return x;
}

fn main() {
    println!("{}", test1());
    println!("{}", test2(13));
    println!("{}", test3());
    println!("{}", minus_one_safe(13));
    println!("{}", minus_one_unsafe(13));
    println!("{}", neg_abs(13));
    println!("{}", abs(13));
}
