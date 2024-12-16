use std::time::{Duration, SystemTime};

use tokio::{task::JoinSet, time::interval};

pub struct TimeTick {
    period: Duration,
    exec_set: JoinSet<()>,
}

impl TimeTick {
    pub fn new(period: Duration) -> TimeTick {
        TimeTick {
            period,
            exec_set: JoinSet::new(),
        }
    }

    pub async fn ticker(&mut self, handler: fn(SystemTime)) {
        let mut delay = interval(self.period);
        loop {
            delay.tick().await;
            self.exec_set.spawn(async move {
                handler(SystemTime::now());
            });
        }
    }
}
