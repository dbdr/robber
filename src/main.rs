fn main() {
	let manifest = std::env::args().nth(1).unwrap_or("".into());
	println!("{}", robber::bundle(&manifest));
}
