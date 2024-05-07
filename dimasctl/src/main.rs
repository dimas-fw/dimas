// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

// region:		--- modules
use clap::{Parser, Subcommand};
use dimas_config::Config;
// endregion:	--- modules

// region:		--- Cli
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct DimasctlArgs {
	#[clap(subcommand)]
	command: DimasctlCommand,
}
// endregion:	--- Cli

// region:		--- Commands
#[derive(Debug, Subcommand)]
enum DimasctlCommand {
	/// List all agents
	List,
}
// endregion:	--- Commands

fn main() {
	let args = DimasctlArgs::parse();

	match &args.command {
		DimasctlCommand::List => {
			let config = Config::default().zenoh_config();
			println!("List of available agents:");
			print!("{}", dimas_commands::list(config));
		}
	}
}
