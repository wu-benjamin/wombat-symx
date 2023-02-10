fn test_float_params(y: f32) -> () {
    assert!(y == 0.0);
}


fn main() {
    test_float_params(1.0);
}