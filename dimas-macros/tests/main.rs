// Copyright © 2024 Stephan Kunz

#[dimas_macros::main(worker_threads)]
async fn main() -> std::result::Result<(), std::io::Error> {
	println!("Hello world");
	Ok(())
}

#[test]
fn main_macro() -> std::result::Result<(), std::io::Error> {
	main()
}
