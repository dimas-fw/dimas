// Copyright © 2024 Stephan Kunz

//! Doku

#[dimas_macros::main(additional_threads = 1)]
async fn main() -> std::result::Result<(), std::io::Error> {
	println!("Hello world");
	Ok(())
}

#[test]
fn main_macro() -> std::result::Result<(), std::io::Error> {
	main()
}
