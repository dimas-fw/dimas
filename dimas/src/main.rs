#![crate_name = "dimas"]
// Copyright © 2023 Stephan Kunz

// region:		--- modules
use clap::Parser;
// endregion:	--- modules

// region:		--- Args
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Name of the person to greet
	#[arg(short, long, value_parser)]
	name: String,

	/// Number of times to greet
	#[arg(short, long, value_parser, default_value_t = 1)]
	count: u8,
}
// endregion:	--- Args

fn main() {
	let args = Args::parse();

	for _ in 0..args.count {
		println!("Hello {}, here is dimas", args.name);
	}
}
