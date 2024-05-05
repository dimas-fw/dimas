// Copyright © 2024 Stephan Kunz

//! Monitoring tool for `DiMAS`

// region:		--- modules
slint::include_modules!();
use clap::Parser;
use std::path::PathBuf;
// endregion:	--- modules

// region:		--- Cli
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional prefix to restrict scope
    prefix: Option<String>,

    /// Use config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

// endregion:	--- Cli

fn main() -> Result<(), slint::PlatformError> {
	// parse args
	let _cli = Cli::parse();

	// create window
	let ui = MainWindow::new()?;

	// implement handlers/callbacks

	// show & run window
	ui.run()
}
