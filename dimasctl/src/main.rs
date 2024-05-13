// Copyright © 2024 Stephan Kunz

//! Commandline tool for `DiMAS`

// region:		--- modules
use clap::{Parser, Subcommand};
use dimas_com::Communicator;
use dimas_config::Config;
use dimas_core::traits::OperationState;
// endregion:	--- modules

// region:		--- Cli
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct DimasctlArgs {
	/// Sets an optional prefix
	#[arg(short, long, value_name = "PREFIX")]
	prefix: Option<String>,

	#[clap(subcommand)]
	command: DimasctlCommand,

	/// An optional zid
	zid: Option<String>,
}
// endregion:	--- Cli

// region:		--- Commands
#[derive(Debug, Subcommand)]
enum DimasctlCommand {
	/// List running `DiMAS` entities
	List,
	/// Scout for `Zenoh` entities
	Scout,
	/// Set state of `Zenoh` entities
	SetState,
}
// endregion:	--- Commands

fn main() {
	let args = DimasctlArgs::parse();
	let config = Config::default();

	let base_selector;

	if let Some(zid) = args.zid.as_deref() {
		base_selector = if let Some(prefix) = args.prefix.as_deref() {
			format!("{}/{}/", prefix, zid)
		} else {
			format!("**/{}/", zid)
		};
	} else {
		base_selector = if let Some(prefix) = args.prefix.as_deref() {
			format!("{}/*/", prefix)
		} else {
			String::from("**/")
		};
	};

	match &args.command {
		// commands that do NOT need a communicator
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
		// commands that need a communicator
		_ => {
			let com = Communicator::new(&config).expect("failed to create 'Communicator'");
			let header = "ZenohId                           Kind    State       Prefix/Name";
			match &args.command {
				DimasctlCommand::List => {
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
				DimasctlCommand::SetState => {
					println!("List of current states of DiMAS entities:");
					println!("{header}");
					for item in
						dimas_commands::set_state(&com, &base_selector, Some(OperationState::Configured))
					{
						println!(
							"{:32}  {:6}  {:10}  {}",
							item.zid(),
							item.kind(),
							item.state(),
							item.name()
						);
					}
				}
				// should be covered by first match level
				_ => {}
			}
		}
	}
}
