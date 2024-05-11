// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

// region:		--- modules
use clap::{Parser, Subcommand};
use dimas_com::Communicator;
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
	/// List running `DiMAS` entities
	List,
	/// Scout for `Zenoh` entities
	Scout,
}
// endregion:	--- Commands

fn main() {
	let args = DimasctlArgs::parse();
	let config = Config::default();

	match &args.command {
		DimasctlCommand::List => {
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			println!("List of running DiMAS entities:");
			println!("ZenohId                           Kind    State       Prefix/Name");
			for item in dimas_commands::about_list(&com) {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state(),
					item.name()
				);
			}
		}
		DimasctlCommand::Scout => {
			println!("List of scouted Zenoh entities:");
			println!("ZenohId                           Kind    Locators");
			for item in dimas_commands::scouting_list(&config) {
				println!(
					"{:32}  {:6}  {:?}",
					item.zid(),
					item.kind(),
					item.locators()
				);
			}
		}
	}
}
