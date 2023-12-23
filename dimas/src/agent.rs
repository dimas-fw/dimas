//! Copyright © 2023 Stephan Kunz

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use zenoh::{
    config::{self, Config},
    queryable::Query,
    sample::Sample,
};

use crate::{com::communicator::Communicator, prelude::*};

// Composable Agent
//#[derive(Debug)]
pub struct Agent<'a> {
    com: Communicator<'a>,
    timer: Vec<Arc<RwLock<Timer<'a>>>>,
}

impl<'a> Agent<'a> {
    pub fn new(config: Config, prefix: impl Into<String>) -> Self {
        let com = Communicator::new(config, prefix);
        Self {
            com,
            timer: Vec::new(),
        }
    }

    pub fn uuid(&self) -> String {
        self.com.uuid()
    }

    pub async fn add_subscriber(&mut self, key_expr: &str, fctn: fn(Sample)) {
        self.com.add_subscriber(key_expr, fctn).await;
    }

    pub async fn add_queryable(&mut self, key_expr: &str, fctn: fn(Query)) {
        self.com.add_queryable(key_expr, fctn).await;
    }

    pub fn add_timer<F>(&mut self, delay: Option<Duration>, repetition: Repetition, fctn: F)
    where
        F: FnMut() + Send + Sync + Unpin + 'static,
    {
        let timer = Arc::new(RwLock::new(Timer::new(delay, repetition, fctn)));
        self.timer.push(timer);
    }

    pub async fn start(&mut self) {
        let timers = self.timer.iter_mut();
        for timer in timers {
            timer.write().unwrap().start().await;
        }
    }
}

impl<'a> Default for Agent<'a> {
    fn default() -> Self {
        Agent::new(config::peer(), "agent")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use serial_test::serial;

    // check, that the auto traits are available
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Agent>();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    //#[serial]
    async fn agent_create() {
        let _agent1 = Agent::default();
        let _agent2 = Agent::new(config::peer(), "agent2");
        //let _agent3 = Agent::new(config::client());
    }
}
