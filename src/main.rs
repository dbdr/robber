fn main() {
	let manifest = std::env::args().nth(1).unwrap_or_else(|| "".into());
	println!("{}", robber::bundle(&manifest));
}
