fn test(
	 c1: bool,
	 c2: bool,
	 c3: bool,
	 c4: bool,
	 c5: bool
) -> i32 {
	let r1 = if c1 {
		-1
	} else {
		0
	};
	let r2 = if c2 {
		2
	} else {
		0
	};
	let r3 = if c3 {
		-4
	} else {
		0
	};
	let r4 = if c4 {
		8
	} else {
		0
	};
	let r5 = if c5 {
		-16
	} else {
		0
	};
	let r = r1 + r2 + r3 + r4 + r5;
	assert!(r >= 0);
	return r;
}
fn main() {
	test(true, false, false, false, false);
}
