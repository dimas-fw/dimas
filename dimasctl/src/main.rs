// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

// region:		--- modules
use clap::{Parser, Subcommand};
use dimas_com::Communicator;
use dimas_config::Config;
use dimas_core::{enums::OperationState, error::Result};
// endregion:	--- modules

// region:		--- Cli
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct DimasctlArgs {
	/// Optional selector for the instances to operate on
	selector: Option<String>,

	#[clap(subcommand)]
	command: DimasctlCommand,
}
// endregion:	--- Cli

fn operation_state_parser(s: &str) -> Result<OperationState> {
	OperationState::try_from(s)
}

// region:		--- Commands
#[derive(Debug, Subcommand)]
enum DimasctlCommand {
	/// List running `DiMAS` entities
	List,
	/// Scout for `Zenoh` entities
	Scout,
	/// Set state of entities
	SetState {
		/// The new state
		#[arg(value_parser = operation_state_parser)]
		state: Option<OperationState>,
	},
	/// Shurdown entities
	Shutdown,
}
// endregion:	--- Commands

fn main() {
	let args = DimasctlArgs::parse();
	let config = Config::default();
	let header = "ZenohId                           Kind    State       Prefix/Name";

	let base_selector = args
		.selector
		.map_or_else(|| String::from("**"), |selector| selector);

	match &args.command {
		DimasctlCommand::List => {
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			println!("List of found DiMAS entities:");
			println!("{header}");
			for item in dimas_commands::about_list(&com, &base_selector) {
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
		DimasctlCommand::SetState { state } => {
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			println!("List of current states of DiMAS entities:");
			println!("{header}");
			for item in dimas_commands::set_state(&com, &base_selector, state.to_owned()) {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state(),
					item.name()
				);
			}
		}
		DimasctlCommand::Shutdown => {
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			println!("List of shut down DiMAS entities:");
			println!("{header}");
			for item in dimas_commands::shutdown(&com, &base_selector) {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state(),
					item.name()
				);
			}
		}
	}
}
