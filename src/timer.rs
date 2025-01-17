use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use ethers::providers::Middleware;
use tokio::{spawn, sync::Mutex, time::interval};

use crate::meantime::MeanTime;

pub struct TimeTick<M> {
    period: Duration,
    mean_time: Arc<Mutex<MeanTime<M>>>,
}

impl<M: Middleware + 'static> TimeTick<M> {
    pub fn new(period: Duration, mean_time: Arc<Mutex<MeanTime<M>>>) -> TimeTick<M> {
        TimeTick {
            period,
            mean_time,
        }
    }

    pub async fn ticker(&mut self) {
        let mut delay = interval(self.period);
        loop {
            delay.tick().await;
            let mean_time = self.mean_time.clone();
            spawn(async move {
                if let Ok(mut mean_time) = mean_time.try_lock() {
                    mean_time.handle_time_tick(SystemTime::now()).await;
                }
            });
        }
    }
}
