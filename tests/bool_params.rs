fn test_bool_params(x: bool) -> () {
    assert!(x || !x);
}


fn main() {
    test_bool_params(true);
}