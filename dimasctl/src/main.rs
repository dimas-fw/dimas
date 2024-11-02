// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

// region:		--- modules
use clap::{Parser, Subcommand};
use core::time::Duration;
use dimas_com::zenoh::Communicator;
use dimas_config::Config;
use dimas_core::{enums::OperationState, Result};
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

fn main() -> Result<()> {
	let args = DimasctlArgs::parse();
	let config = Config::default();
	let h_zid = "ZenohId";
	let h_kind = "Kind";
	let h_state = "State";
	let h_name = "Prefix/Name";

	let base_selector = args
		.selector
		.clone()
		.map_or_else(|| String::from("**"), |selector| selector);

	match &args.command {
		DimasctlCommand::List => {
			let com =
				Communicator::new(config.zenoh_config()).expect("failed to create 'Communicator'");
			println!("List of found DiMAS entities:");
			println!("{h_zid:32}  {h_kind:6}  {h_state:10}  {h_name}");
			let list = dimas_commands::about_list(&com, &base_selector)?;
			for item in list {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state().to_string(),
					item.name()
				);
			}
		}
		DimasctlCommand::Ping { target, count } => {
			let target = args
				.selector
				.map_or_else(|| target.to_owned(), |value| format!("{value}/{target}"));
			let com =
				Communicator::new(config.zenoh_config()).expect("failed to create 'Communicator'");
			for _ in 0..*count {
				let list = dimas_commands::ping_list(&com, &target)?;
				for item in list {
					#[allow(clippy::cast_precision_loss)]
					let time = item.1 as f64 / 2_000_000.0;
					println!("{:32}  {:6.2}ms  {}", item.0.zid(), time, item.0.name(),);
				}
				if *count > 1 {
					println!("\r");
					std::thread::sleep(Duration::from_millis(1000));
				}
			}
		}
		DimasctlCommand::Scout => {
			println!("List of scouted Zenoh entities:");
			println!("ZenohId                           Kind    Locators");
			let list = dimas_commands::scouting_list(&config)?;
			for item in list {
				println!(
					"{:32}  {:6}  {:?}",
					item.zid(),
					item.kind(),
					item.locators()
				);
			}
		}
		DimasctlCommand::SetState { state } => {
			let com =
				Communicator::new(config.zenoh_config()).expect("failed to create 'Communicator'");
			println!("List of current states of DiMAS entities:");
			println!("{h_zid:32}  {h_kind:6}  {h_state:10}  {h_name}");
			let list = dimas_commands::set_state(&com, &base_selector, state.to_owned())?;
			for item in list {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state().to_string(),
					item.name()
				);
			}
		}
		DimasctlCommand::Shutdown { target } => {
			let target = args
				.selector
				.map_or_else(|| target.to_owned(), |value| format!("{value}/{target}"));
			let com =
				Communicator::new(config.zenoh_config()).expect("failed to create 'Communicator'");
			println!("List of shut down DiMAS entities:");
			println!("{h_zid:32}  {h_kind:6}  {h_state:10}  {h_name}");
			let list = dimas_commands::shutdown(&com, &target)?;
			for item in list {
				println!(
					"{:32}  {:6}  {:10}  {}",
					item.zid(),
					item.kind(),
					item.state().to_string(),
					item.name()
				);
			}
		}
	}
	Ok(())
}
