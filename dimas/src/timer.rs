//! Copyright © 2023 Stephan Kunz

use std::{time::Duration, sync::Arc};

use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Debug, Clone)]
pub enum Repetition {
	Count(i32),
	Interval(Duration),
}

//#[derive(Debug)]
pub struct Timer {
	delay: Option<Duration>,
	repetition: Repetition,
	fctn: Arc<Mutex<dyn FnMut() + Send + Sync + Unpin + 'static>>,
	started: bool,
}

impl Timer {
	pub fn new<F>(delay: Option<Duration>, repetition: Repetition, fctn: F) -> Timer
	where
		F: FnMut() + Send + Sync + Unpin + 'static,
	{
		Timer {
			delay,
			repetition,
			fctn: Arc::new(Mutex::new(fctn)),
			started: false,
		}
	}

	pub fn start(&mut self) -> Option<JoinHandle<()>> {
		if !self.started {
			self.started = true;
			let delay = self.delay;
			let repetition = self.repetition.clone();
			let fctn = self.fctn.clone();
			Some(tokio::spawn(async move {
				if delay.is_some() {
					tokio::time::sleep(delay.unwrap()).await;
				}
				match repetition {
					Repetition::Count(number) => {
						for _i in 0..number {
							let mut fctn = fctn.lock().await;
							fctn();
						}
					}
					Repetition::Interval(interval) => loop {
						let mut fctn = fctn.lock().await;
						fctn();
						tokio::time::sleep(interval).await;
					},
				}
			}))
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// check, that the auto traits are available
	fn is_normal<T: Sized + Send + Sync + Unpin>() {}

	#[test]
	fn normal_types() {
		is_normal::<Timer>();
	}

	fn test() {
		dbg!("Test");
	}

	#[test]
	fn timer_create() {
		let _timer1 = Timer::new(Some(Duration::from_millis(100)), Repetition::Count(1), test);
		let _timer2 = Timer::new(
			Some(Duration::from_millis(100)),
			Repetition::Interval(Duration::from_millis(100)),
			|| {
				dbg!("Test");
			},
		);
	}

	#[tokio::test]
	async fn timer_start() {
		let mut timer = Timer::new(Some(Duration::from_millis(100)), Repetition::Count(5), test);
		let h = timer.start().unwrap();
		let res = tokio::join!(h);
		match res {
    	(Ok(_),) => (),
    	(Err(_),) => panic!("join error"),
		}
	}
}
