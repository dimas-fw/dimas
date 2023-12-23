//! Copyright © 2023 Stephan Kunz

use std::time::Duration;

#[derive(Debug)]
pub enum Repetition {
    Count(i32),
    Interval(Duration),
}

//#[derive(Debug)]
pub struct Timer<'a> {
    delay: Option<Duration>,
    repetition: Repetition,
    fctn: Box<dyn FnMut() + Send + Sync + Unpin + 'a>, //fn(),
    started: bool,
}

impl<'a> Timer<'a> {
    pub fn new<F>(delay: Option<Duration>, repetition: Repetition, fctn: F) -> Timer<'a>
    where
        F: FnMut() + Send + Sync + Unpin + 'a,
    {
        Timer {
            delay,
            repetition,
            fctn: Box::new(fctn),
            started: false,
        }
    }

    pub async fn start(&mut self) {
        if !self.started {
            self.started = true;
            if self.delay.is_some() {
                let delay = self.delay.unwrap();
                tokio::time::sleep(delay).await;
            }
            match self.repetition {
                Repetition::Count(number) => {
                    for _i in 0..number {
                        (self.fctn)();
                    }
                }
                Repetition::Interval(interval) => loop {
                    (self.fctn)();
                    tokio::time::sleep(interval).await;
                },
            }
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
        timer.start().await;
    }
}
