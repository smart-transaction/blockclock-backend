use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use ethers::providers::Middleware;
use tokio::{sync::Mutex, task::JoinSet, time::interval};

use crate::meantime::MeanTime;

pub struct TimeTick<M> {
    period: Duration,
    exec_set: JoinSet<()>,
    mean_time: Arc<Mutex<MeanTime<M>>>,
}

impl<M: Middleware + 'static> TimeTick<M> {
    pub fn new(period: Duration, mean_time: Arc<Mutex<MeanTime<M>>>) -> TimeTick<M> {
        TimeTick {
            period,
            exec_set: JoinSet::new(),
            mean_time,
        }
    }

    pub async fn ticker(&mut self) {
        let mut delay = interval(self.period);
        loop {
            delay.tick().await;
            let mean_time = self.mean_time.clone();
            self.exec_set.spawn(async move {
                if let Ok(mut mean_time) = mean_time.try_lock() {
                    mean_time.handle_time_tick(SystemTime::now()).await;
                }
            });
        }
    }
}
