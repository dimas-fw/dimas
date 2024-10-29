//! `DiMAS` query example
//! Copyright © 2024 Stephan Kunz

use dimas::prelude::*;

#[derive(Debug)]
struct AgentProps {}

async fn query_callback(_ctx: Context<AgentProps>, response: QueryableMsg) -> Result<()> {
	let message: u128 = response.decode()?;
	println!("Response 1 is '{message}'");
	Ok(())
}

#[dimas::main]
async fn main() -> Result<()> {
	// create & initialize agents properties
	let properties = AgentProps {};

	// create an agent with the properties and the prefix 'examples'
	let agent = Agent::new(properties)
		.prefix("examples")
		.name("querier")
		.config(&Config::default())?;

	// create querier for topic "query"
	agent
		.querier()
		.topic("query")
		.callback(query_callback)
		.add()?;

	// timer for regular querying
	let interval = Duration::from_secs(1);
	let mut counter1 = 0i128;
	agent
		.timer()
		.name("timer")
		.interval(interval)
		.callback(move |ctx| -> Result<()> {
			println!("Querying [{counter1}]");
			let message = Message::encode(&counter1);
			// querying with stored query
			ctx.get("query", Some(message), None)?;
			counter1 += 1;
			Ok(())
		})
		.add()?;

	// run agent
	agent.start().await?;

	Ok(())
}
