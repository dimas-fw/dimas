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
			println!("{:32}  {:6}", "ZenohId", "Kind");
			for item in dimas_commands::DimasEntity::fetch(&config) {
				println!("{:32}  {:6}", item.zid(), item.kind());
			}
		}
	}
}
