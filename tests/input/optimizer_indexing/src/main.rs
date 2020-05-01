fn main() {
	// 1D array
	let mut a = [1, 30];
	a[1] = 3;
	let index = a[1];
	assert_eq!(a[1], 3);
	
	// 2D array
	let mut data = [[0; 2]; 2];
	// test assignment
	data[0][index] = 41;
	// test op_assignment
	data[0][index] += 1;
	
	// rvalue
	let answer = data[1][1];
	//TODO let answer = data[0][index];
	
	assert_eq!(answer, 42);

	println!("Hello, CodinGame!\n{}", answer);
}
