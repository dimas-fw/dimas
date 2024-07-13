// Copyright © 2024 Stephan Kunz

use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use bitcode::encode;
// region:		--- modules
use super::{
	publisher::Publisher, ArcObservableControlCallback, ArcObservableExecutionFunction,
	ArcObservableFeedbackCallback,
};
use crate::timer::Timer;
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::{DimasError, Result},
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::{Capability, Context, ContextAbstraction},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::session::SessionDeclarations;
use zenoh::Wait;
use zenoh::{
	qos::{CongestionControl, Priority},
	sample::Locality,
};
// endregion:	--- modules

// region:		--- Observable
/// Observable
pub struct Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// The observables key expression
	selector: String,
	/// Context for the Observable
	context: Context<P>,
	activation_state: OperationState,
	feedback_interval: Duration,
	/// callback for observation request and cancelation
	control_callback: ArcObservableControlCallback<P>,
	/// callback for observation feedback
	feedback_callback: ArcObservableFeedbackCallback<P>,
	/// function for observation execution
	execution_function: ArcObservableExecutionFunction<P>,
	handle: Option<JoinHandle<()>>,
}

impl<P> std::fmt::Debug for Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Observable")
			.finish_non_exhaustive()
	}
}

impl<P> Capability for Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn manage_operation_state(&mut self, state: &OperationState) -> Result<()> {
		if (state >= &self.activation_state) && self.handle.is_none() {
			return self.start();
		} else if (state < &self.activation_state) && self.handle.is_some() {
			self.stop();
			return Ok(());
		}
		Ok(())
	}
}

impl<P> Observable<P>
where
	P: Send + Sync + Unpin + 'static,
{
	/// Constructor for an [`Observable`]
	#[must_use]
	pub fn new(
		selector: String,
		context: Context<P>,
		activation_state: OperationState,
		feedback_interval: Duration,
		control_callback: ArcObservableControlCallback<P>,
		feedback_callback: ArcObservableFeedbackCallback<P>,
		execution_function: ArcObservableExecutionFunction<P>,
	) -> Self {
		Self {
			selector,
			context,
			activation_state,
			feedback_interval,
			control_callback,
			feedback_callback,
			execution_function,
			handle: None,
		}
	}

	/// Get `selector`
	#[must_use]
	pub fn selector(&self) -> &str {
		&self.selector
	}

	/// Start or restart the Observable.
	/// An already running Observable will be stopped, eventually damaged Mutexes will be repaired
	#[instrument(level = Level::TRACE, skip_all)]
	fn start(&mut self) -> Result<()> {
		self.stop();

		{
			if self.control_callback.lock().is_err() {
				warn!("found poisoned put Mutex");
				self.control_callback.clear_poison();
			}
		}

		let selector = self.selector.clone();
		let interval = self.feedback_interval;
		let ccb = self.control_callback.clone();
		let fcb = self.feedback_callback.clone();
		let efc = self.execution_function.clone();
		let ctx1 = self.context.clone();
		let ctx2 = self.context.clone();

		self.handle
			.replace(tokio::task::spawn(async move {
				let key = selector.clone();
				std::panic::set_hook(Box::new(move |reason| {
					error!("queryable panic: {}", reason);
					if let Err(reason) = ctx1
						.sender()
						.send(TaskSignal::RestartObservable(key.clone()))
					{
						error!("could not restart observable: {}", reason);
					} else {
						info!("restarting observable!");
					};
				}));
				if let Err(error) = run_observable(selector, interval, ccb, fcb, efc, ctx2).await {
					error!("observable failed with {error}");
				};
			}));

		Ok(())
	}

	/// Stop a running Observable
	#[instrument(level = Level::TRACE, skip_all)]
	fn stop(&mut self) {
		if let Some(handle) = self.handle.take() {
			handle.abort();
		}
	}
}
// endregion:	--- Observable

// region:		--- functions
#[instrument(name="observable", level = Level::ERROR, skip_all)]
async fn run_observable<P>(
	selector: String,
	feedback_interval: Duration,
	control_callback: ArcObservableControlCallback<P>,
	feedback_callback: ArcObservableFeedbackCallback<P>,
	execution_function: ArcObservableExecutionFunction<P>,
	ctx: Context<P>,
) -> Result<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let queryable = ctx
		.session()
		.declare_queryable(&selector)
		.complete(true)
		.allowed_origin(Locality::Any)
		.await?;

	let mut executor_handle: Option<Arc<Mutex<JoinHandle<()>>>> = None;
	let mut feedback_handle: Option<JoinHandle<()>> = None;
	let mut response_publisher: Option<Arc<Mutex<Publisher<P>>>> = None;

	loop {
		let feedback_callback = feedback_callback.clone();
		let execution_function = execution_function.clone();
		let query = queryable
			.recv_async()
			.await
			.map_err(|_| DimasError::ShouldNotHappen)?;

		let p = query.parameters().as_str();
		// TODO: make a proper "key: value" implementation
		if p == "request" {
			let content = query.payload().map_or_else(
				|| {
					let content: Vec<u8> = Vec::new();
					content
				},
				|value| {
					let content: Vec<u8> = value.into();
					content
				},
			);
			let msg = Message::new(content);
			match control_callback.lock() {
				Ok(mut lock) => {
					let res = lock(&ctx, msg);
					match res {
						Ok(response) => {
							match response {
								ControlResponse::Accepted => {
									let key = query.selector().key_expr.to_string();
									let publisher_selector =
										format!("{}/feedback/{}", &key, ctx.session().zid());

									let mut publisher = Publisher::new(
										publisher_selector,
										ctx.clone(),
										OperationState::Created,
										Priority::RealTime,
										CongestionControl::Block,
									);
									publisher
										.manage_operation_state(&OperationState::Active)
										.expect("snh");

									let publisher = Arc::new(Mutex::new(publisher));
									response_publisher.replace(publisher.clone());

									// start executor
									let executor = Arc::new(Mutex::new(start_executor(
										publisher.clone(),
										execution_function,
										ctx.clone(),
									)));
									executor_handle.replace(executor.clone());
									feedback_handle.replace(start_feedback(
										publisher.clone(),
										feedback_interval,
										feedback_callback,
										ctx.clone(),
										executor,
									));
									// send accepted response
									let encoded: Vec<u8> = encode(&ControlResponse::Accepted);
									query
										.reply(&key, encoded)
										.wait()
										.map_err(|_| DimasError::ShouldNotHappen)?;
								}
								ControlResponse::Declined => {
									// send declined response
									let key = query.selector().key_expr.to_string();
									let encoded: Vec<u8> = encode(&ControlResponse::Declined);
									query
										.reply(&key, encoded)
										.wait()
										.map_err(|_| DimasError::ShouldNotHappen)?;
								}
								ControlResponse::Canceled => todo!(),
							}
						}
						Err(error) => error!("observable cotrol callback failed with {error}"),
					}
				}
				Err(err) => {
					error!("observable control callback failed with {err}");
				}
			}
		} else if p == "cancel" {
			// stop running observation
			if let Some(handle) = feedback_handle.take() {
				handle.abort();
			};
			if let Some(handle) = executor_handle.take() {
				handle
					.lock()
					.map_or_else(|_| {}, |handle| handle.abort());
			};

			// send canceled resultresponse
			if let Some(publisher) = response_publisher.clone() {
				let msg = match feedback_callback.lock() {
					Ok(mut fcb) => fcb(&ctx)?,
					Err(_) => todo!(),
				};
				let response = ObservableResponse::Canceled(msg.value().clone());
				match publisher.lock() {
					Ok(lock) => {
						let _res = lock.put(Message::encode(&response));
					}
					Err(reason) => {
						error!("could not publish cancelation: {}", reason);
					}
				}
			};
			// send canceled control response
			let encoded: Vec<u8> = encode(&ControlResponse::Canceled);
			let key = query.selector().key_expr.to_string();
			query
				.reply(&key, encoded)
				.wait()
				.map_err(|_| DimasError::ShouldNotHappen)?;
		} else {
			error!("observable got unknown parameter: {p}");
		}
	}
}

fn start_feedback<P>(
	publisher: Arc<Mutex<Publisher<P>>>,
	feedback_interval: Duration,
	feedback_callback: ArcObservableFeedbackCallback<P>,
	ctx: Context<P>,
	executor_handle: Arc<Mutex<JoinHandle<()>>>,
) -> JoinHandle<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let handle = tokio::task::spawn(async move {
		let mut timer = Timer::new(
			"feedback".to_string(),
			ctx,
			OperationState::Created,
			Arc::new(Mutex::new(
				move |ctx: &Arc<dyn ContextAbstraction<P>>| -> Result<()> {
					match executor_handle.lock() {
						Ok(handle) => {
							if !handle.is_finished() {
								match feedback_callback.lock() {
									Ok(mut fcb) => {
										let msg = fcb(ctx)?;
										let response =
											ObservableResponse::Feedback(msg.value().clone());
										match publisher.lock() {
											Ok(lock) => {
												let _res = lock.put(Message::encode(&response));
											}
											Err(reason) => {
												error!("could not publish feedback: {}", reason);
											}
										}
									}
									Err(_) => todo!(),
								}
							}
						}
						Err(_) => todo!(),
					};
					Ok(())
				},
			)),
			feedback_interval,
			Some(Duration::from_millis(5)),
		);
		timer
			.manage_operation_state(&OperationState::Active)
			.expect("snh");
	});
	handle
}

fn start_executor<P>(
	publisher: Arc<Mutex<Publisher<P>>>,
	execution_function: ArcObservableExecutionFunction<P>,
	ctx: Context<P>,
) -> JoinHandle<()>
where
	P: Send + Sync + Unpin + 'static,
{
	let handle = tokio::task::spawn(async move {
		match execution_function.lock() {
			Ok(mut cb) => {
				let res = cb(&ctx);
				match res {
					Ok(msg) => {
						let response = ObservableResponse::Finished(msg.value().clone());
						match publisher.lock() {
							Ok(lock) => {
								if let Err(reason) = lock.put(Message::encode(&response)) {
									error!("could not publish result: {}", reason);
								}
							}
							Err(reason) => {
								error!("could not publish result: {}", reason);
							}
						};
					}
					Err(error) => error!("execution function failed with {error}"),
				}
			}
			Err(err) => {
				error!("execution function lock failed with {err}");
			}
		}
	});
	handle
}
// endregion:	--- functions

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug)]
	struct Props {}

	// check, that the auto traits are available
	const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	const fn normal_types() {
		is_normal::<Observable<Props>>();
	}
}
