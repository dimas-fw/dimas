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
	message_types::{ControlResponse, Message},
	traits::{Capability, Context, ContextAbstraction},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::sample::Locality;
use zenoh::session::SessionDeclarations;
use zenoh::{
	core::{Priority, Wait},
	publisher::CongestionControl,
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
	/// fuction for observation execution
	execution_function: ArcObservableExecutionFunction<P>,
	/// inner data
	rt_data: Arc<Mutex<ObservableData<P>>>,
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
	// helper for handling runtime data
	let rt_data = Arc::new(Mutex::new(ObservableData {
		publisher: None,
		timer: None,
		execution: None,
	}));
		Self {
			selector,
			context,
			activation_state,
			feedback_interval,
			control_callback,
			feedback_callback,
			execution_function,
			rt_data,
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
		let rt_data = self.rt_data.clone();

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
				if let Err(error) = run_observable(selector, rt_data, interval, ccb, fcb, efc, ctx2).await {
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

// region:		--- ObservableData
struct ObservableData<P>
where
	P: Send + Sync + Unpin + 'static,
{
	publisher: Option<Arc<Mutex<Publisher<P>>>>,
	timer: Option<Timer<P>>,
	execution: Option<JoinHandle<()>>,
}

impl<P> ObservableData<P>
where
	P: Send + Sync + Unpin + 'static,
{
	fn create_publisher(
		&mut self,
		key: &String,
		feedback_interval: Duration,
		feedback_callback: ArcObservableFeedbackCallback<P>,
		ctx: Context<P>,
	) -> Result<()> {
		let publisher_selector = format!("{}/feedback/{}", &key, ctx.session().zid());

		let mut publisher = Publisher::new(
			publisher_selector,
			ctx.clone(),
			OperationState::Created,
			Priority::RealTime,
			CongestionControl::Block,
		);
		publisher.manage_operation_state(&OperationState::Active)?;

		let arc_publisher = Arc::new(Mutex::new(publisher));
		let publisher = arc_publisher.clone();
		self.publisher = Some(arc_publisher);

		let mut timer = Timer::new(
			"feedback".to_string(),
			ctx,
			OperationState::Created,
			Arc::new(Mutex::new(
				move |ctx: &Arc<dyn ContextAbstraction<P>>| -> Result<()> {
					match feedback_callback.lock() {
						Ok(mut fcb) => {
							let msg = fcb(ctx)?;
							match publisher.lock() {
								Ok(lock) => lock.put(msg),
								Err(reason) => {
									error!("could not publish feedback: {}", reason);
									Ok(())
								}
							}
						}
						Err(_) => todo!(),
					}
				},
			)),
			feedback_interval,
			Some(feedback_interval),
		);
		timer.manage_operation_state(&OperationState::Active)?;
		self.timer.replace(timer);

		Ok(())
	}

	fn create_executor(
		&mut self,
		key: String,
		execution_function: ArcObservableExecutionFunction<P>,
		ctx: Context<P>,
		rt_data: Arc<Mutex<Self>>,
	) {
		let ctx2 = ctx.clone();
		let publisher = self.publisher.take();

		self
		.execution
		.replace(tokio::task::spawn(async move {
			std::panic::set_hook(Box::new(move |reason| {
				error!("observable execution panic: {}", reason);
				if let Err(reason) = ctx.sender().send(
					TaskSignal::RestartObservable(key.clone()),
				) {
					error!(
						"could not restart observable: {}",
						reason
					);
				} else {
					info!("restarting observable!");
				};
			}));
			let publisher2 = publisher.expect("snh");
			match execution_function.lock() {
				Ok(mut cb) => {
					let res = cb(&ctx2);
					match res {
						Ok(result) => {
							let msg = Message::encode(&result);
							match publisher2.lock() {
								Ok(lock) => {
									if let Err(reason) =
										lock.put(msg)
									{
										error!("could not publish result: {}", reason);
									}
								}
								Err(reason) => {
									error!(
										"could not publish result: {}",
										reason
									);
								}
							};
							match rt_data.lock() {
								Ok(mut lock) => {
									lock.execution.take();
									lock.timer.take();
									lock.publisher.take();
								},
								Err(error) => error!("inner lock failed with {error}"),
							};
						}
						Err(error) => error!(
						"execution function failed with {error}"
						),
					}
				}
				Err(err) => {
					error!(
						"execution function lock failed with {err}"
					);
				}
			}
		}));
	}
}
// endregion:	--- ObservableData

#[instrument(name="observable", level = Level::ERROR, skip_all)]
async fn run_observable<P>(
	selector: String,
	rt_data: Arc<Mutex<ObservableData<P>>>,
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

	loop {
		let feedback_callback = feedback_callback.clone();
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
									match rt_data.lock() {
										Ok(mut lock) => {
											// create and start feedback publisher
											let key = query.selector().key_expr.to_string();
											lock.create_publisher(&key, feedback_interval.clone(), feedback_callback.clone(), ctx.clone())?;

											// run task
											lock.create_executor(key.clone(), execution_function.clone(), ctx.clone(), rt_data.clone());

											// send accepted response
											let encoded: Vec<u8> = encode(&ControlResponse::Accepted);
											query
												.reply(&key, encoded)
												.wait()
												.map_err(|_| DimasError::ShouldNotHappen)?;
										},
										Err(_) => todo!(),
									};
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
						Err(error) => error!("observable callback failed with {error}"),
					}
				}
				Err(err) => {
					error!("observable callback failed with {err}");
				}
			}
		} else if p == "cancel" {
			let todo = true;
			// stop running observation
			match rt_data.lock() {
				Ok(mut lock) => {
					lock.execution.take();
					lock.timer.take();
					lock.publisher.take();
				},
				Err(error) => error!("inner lock failed with {error}"),
			};

			// send canceled response
			let state = Message::encode(&"canceled".to_string());
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
