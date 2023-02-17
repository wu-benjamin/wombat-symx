fn test(
	c1: bool,
	// c2: bool
) -> i32 {
	let mut r = 0;
	if c1 {
		r -= 1;
	}
	// if c2 {
	// 	r += 2;
	// }
	assert!(r >= 0);
	return r;
}

fn main() {
	test(true);
}