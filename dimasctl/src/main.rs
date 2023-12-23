#![crate_name = "dimasctl"]
// Copyright © 2023 Stephan Kunz

use clap::Parser;

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

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}, here is dimasctl", args.name);
    }
}
