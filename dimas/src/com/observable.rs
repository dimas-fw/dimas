// Copyright © 2024 Stephan Kunz

use std::time::Duration;

use bitcode::encode;
// region:		--- modules
use super::{
	ArcObservableControlCallback, ArcObservableExecutionFunction, ArcObservableFeedbackCallback,
};
use dimas_core::{
	enums::{OperationState, TaskSignal},
	error::Result,
	message_types::{ControlResponse, Message, ObservableResponse},
	traits::{Capability, Context},
};
use tokio::task::JoinHandle;
use tracing::{error, info, instrument, warn, Level};
use zenoh::Wait;
use zenoh::{qos::QoSBuilderTrait, session::SessionDeclarations};
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

		// check Mutexes
		{
			if self.control_callback.lock().is_err() {
				warn!("found poisoned control Mutex");
				self.control_callback.clear_poison();
			}
		}

		{
			if self.feedback_callback.lock().is_err() {
				warn!("found poisoned feedback Mutex");
				self.feedback_callback.clear_poison();
			}
		}

		//{
		//	if self.execution_function.lock().is_err() {
		//		warn!("found poisoned execution Mutex");
		//		self.execution_function.clear_poison();
		//	}
		//}

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
						.blocking_send(TaskSignal::RestartObservable(key.clone()))
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
	// create the control queryable
	let queryable = ctx
		.session()
		.declare_queryable(&selector)
		.complete(true)
		.allowed_origin(Locality::Any)
		.await?;

	// initialize a pinned feedback timer
	// TODO: init here leads to on unnecessary timer-cycle without doing something
	let feedback_timer = tokio::time::sleep(feedback_interval);
	tokio::pin!(feedback_timer);

	// base communication key & selector for feedback publisher
	let key = selector.clone();
	let publisher_selector = format!("{}/feedback/{}", &key, ctx.session().zid());

	// variables to manage control loop
	let mut is_running = false;
	let (tx, mut rx) = tokio::sync::mpsc::channel(8);
	let mut execution_handle: Option<JoinHandle<()>> = None;
	let mut feedback_publisher: Option<zenoh::pubsub::Publisher> = None;

	// main control loop of the observable
	// started and terminated by state management
	// do not terminate loop in case of errors during execution
	loop {
		// different cases that may happen
		tokio::select! {
			// got query from an observer
			Ok(query) = queryable.recv_async() => {
				// TODO: make a proper "key: value" implementation
				let p = query.parameters().as_str();
				if p == "request" {
					// received request => if no execution is running: spawn execution with channel for result else: return already running message
					if is_running {
						// send occupied response
						let key = query.selector().key_expr.to_string();
						let encoded: Vec<u8> = encode(&ControlResponse::Occupied);
						match query.reply(&key, encoded).wait() {
							Ok(()) => {},
							Err(err) => error!("failed to reply with {err}"),
						};
					} else {
						// start a computation
						// create Message from payload
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
						// check whether request is possible using control callback
						match control_callback.lock() {
							Ok(mut lock) => {
								let res = lock(&ctx, msg);
								match res {
									Ok(response) => {
										if matches!(response, ControlResponse::Accepted ) {
											// create feedback publisher
											ctx.session()
												.declare_publisher(publisher_selector.clone())
												.congestion_control(CongestionControl::Block)
												.priority(Priority::RealTime)
												.wait()
												.map_or_else(
													|err| error!("could not create feedback publisher due to {err}"),
													|publ| { feedback_publisher.replace(publ); }
												);


											// spawn execution
											let tx_clone = tx.clone();
											let execution_function_clone = execution_function.clone();
											let ctx_clone = ctx.clone();
											execution_handle.replace(tokio::spawn( async move {
												let res = execution_function_clone.lock().await(&ctx_clone).map_or_else(
													|_| { todo!() },
													|res| { res }
												);
												if !matches!(tx_clone.send(res).await, Ok(())) { error!("failed to send back execution result") };
											}));

											// start feedback timer
											feedback_timer.set(tokio::time::sleep(feedback_interval));
											is_running = true;
										}
										// send  response back to requestor
										let encoded: Vec<u8> = encode(&response);
										match query.reply(&key, encoded).wait() {
											Ok(()) => {},
											Err(err) => error!("failed to reply with {err}"),
										};
									}
									Err(error) => error!("control callback failed with {error}"),
								}
							}
							Err(error) => error!("control callback failed with {error}"),
						}
					}
				} else if p == "cancel" {
					// received cancel => abort a running execution
					if is_running {
						is_running = false;
						let publisher = feedback_publisher.take();
						let handle = execution_handle.take();
						if let Some(h) = handle { 
							h.abort();
							// wait for abortion
							let _ = h.await;
							// send cancelation feedback with last state
							match feedback_callback.lock() {
								Ok(mut fcb) => {
									let Ok(msg) = fcb(&ctx) else { todo!() };
									let response =
										ObservableResponse::Canceled(msg.value().clone());
									if let Some(p) = publisher {
										match p.put(Message::encode(&response).value().clone()).wait() {
											Ok(()) => {},
											Err(err) => error!("could not send result due to {err}"),
										};
									} else {
										error!("missing publisher");
									};
								}
								Err(_) => { todo!() },
							};
						} else { 
							error!("unexpected absence of join handle");
						};
					}
					// acknowledge cancel request
					let encoded: Vec<u8> = encode(&ControlResponse::Canceled);
					match query.reply(&key, encoded).wait() {
						Ok(()) => {},
						Err(err) => error!("failed to reply with {err}"),
					};
				} else {
					error!("observable got unknown parameter: {p}");
				}
			}

			// request finished => send back result of request (which may be a failure)
			Some(result) = rx.recv() => {
				if is_running {
					is_running = false;
					execution_handle.take();
					let response = ObservableResponse::Finished(result.value().clone());
					feedback_publisher.take().map_or_else(
						|| error!("could not publish result"),
						|p| {
							match p.put(Message::encode(&response).value()).wait() {
								Ok(()) => {},
								Err(err) => error!("publishing result failed due to {err}"),
							};
						}
					);
				}
			}

			// feedback timer expired and observable still is executing
			() = &mut feedback_timer, if is_running => {
				// send feedback
				match feedback_callback.lock() {
					Ok(mut fcb) => {
						let Ok(msg) = fcb(&ctx) else { todo!() };
						let response =
							ObservableResponse::Feedback(msg.value().clone());

						let publisher = feedback_publisher.as_ref().map_or_else(
							|| { todo!() },
							|p| p
						);
						match publisher.put(Message::encode(&response).value().clone()).wait() {
							Ok(()) => {},
							Err(err) => error!("publishing feedback failed due to {err}"),
						};
					}
					Err(_) => { todo!() },
				}

				// restart timer
				feedback_timer.set(tokio::time::sleep(feedback_interval));
			}
		}
	}
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
