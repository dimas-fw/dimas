// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

use std::{thread, time::Duration};

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
	/// Ping entities
	Ping {
		/// Selector for the targets to ping
		/// - will be concatenated with optional selector
		target: String,
		/// An optional number of ping repetitiions
		#[arg(short, long, default_value = "1")]
		count: u8,
	},
	/// Scout for `Zenoh` entities
	Scout,
	/// Set state of entities
	SetState {
		/// The new state
		#[arg(value_parser = operation_state_parser)]
		state: Option<OperationState>,
	},
	/// Shurdown entities
	Shutdown {
		/// Selector for the targets to shutdown
		/// - will be concatenated with optional selector
		target: String,
	},
}
// endregion:	--- Commands

fn main() {
	let args = DimasctlArgs::parse();
	let config = Config::default();
	let header = "ZenohId                           Kind    State       Prefix/Name";

	let base_selector = args
		.selector
		.clone()
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
		DimasctlCommand::Ping { target, count } => {
			let target = args
				.selector
				.map_or_else(|| target.to_owned(), |value| format!("{value}/{target}"));
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			for _ in 0..*count {
				for item in dimas_commands::ping_list(&com, &target) {
					#[allow(clippy::cast_precision_loss)]
					let time = item.1 as f64 / 2_000_000.0;
					println!("{:32}  {:6.2}ms  {}", item.0.zid(), time, item.0.name(),);
				}
				if *count > 1 {
					println!("\r");
					thread::sleep(Duration::from_millis(1000));
				}
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
		DimasctlCommand::Shutdown { target } => {
			let target = args
				.selector
				.map_or_else(|| target.to_owned(), |value| format!("{value}/{target}"));
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			println!("List of shut down DiMAS entities:");
			println!("{header}");
			for item in dimas_commands::shutdown(&com, &target) {
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
