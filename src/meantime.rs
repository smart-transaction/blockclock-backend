use std::{sync::Arc, time::Duration};

use ethers::{
    providers::Middleware,
    types::{Address, Bytes, U256},
    utils::parse_units,
};
use tokio::sync::Mutex;

use crate::{
    time_pool::TimeSigPool,
    time_signature::{BlockTime, Chronicle},
};

pub struct MeanTime<M> {
    pool: Arc<Mutex<TimeSigPool>>,
    block_time_contract: BlockTime<M>,
    time_window: Duration,
    last_sig: Chronicle,
    mean_time_guard: Mutex<bool>,
}

impl<M: Middleware> MeanTime<M> {
    pub fn new(
        pool: Arc<Mutex<TimeSigPool>>,
        block_time_address: Address,
        middleware: Arc<M>,
        time_window: Duration,
    ) -> MeanTime<M> {
        MeanTime {
            pool,
            block_time_contract: BlockTime::new(block_time_address, middleware),
            time_window,
            last_sig: Chronicle::new(0.into(), Address::zero(), Bytes::new()),
            mean_time_guard: Mutex::new(true),
        }
    }

    pub async fn send_mean_time(&mut self, curr_ts: Duration) {
        let res = self.mean_time_guard.try_lock();
        if let Err(_) = res {
            // The mean time computation and sending is in progress, don't start another one
            return;
        }
        // Get mean time
        if let Some((mean_time, last_sigs)) = self.compute_mean_time(curr_ts).await {
            self.last_sig = last_sigs.last().unwrap().clone();
            // Send the mean time and signatures to the contract
            let receivers: Vec<Address> = last_sigs
                .clone()
                .into_iter()
                .map(|el| el.time_keeper)
                .collect();
            let amount: U256 = parse_units(1, "ether").ok().unwrap().into();
            let amounts: Vec<U256> = vec![amount; receivers.len()];
            match self
                .block_time_contract
                .move_time(last_sigs, mean_time, receivers, amounts)
                .gas(10000000)
                .send()
                .await
            {
                Ok(pending) => {
                    println!("Transaction is sent, txhash: {}", pending.tx_hash());
                    match pending.await {
                        Ok(receipt) => {
                            if let Some(receipt) = receipt {
                                if let Some(status) = receipt.status {
                                    println!("Got transaction status: {}", status);
                                    return;
                                }
                            }
                            println!("Transaction status wasn't received.");
                            return;
                        }
                        Err(err) => {
                            println!("Error pending transaction: {}", err);
                            return;
                        }
                    }
                }
                Err(err) => {
                    println!("Error sending transaction: {}", err);
                }
            }
        }
    }

    async fn compute_mean_time(&self, curr_ts: Duration) -> Option<(U256, Vec<Chronicle>)> {
        // Check the latest signature.
        let mut pool = self.pool.lock().await;
        if pool.is_empty() {
            return None;
        }
        pool.sort_by_key(|el| el.epoch);
        if let Some(last_sig) = pool.last() {
            if last_sig.signature == self.last_sig.signature {
                // No new time, no need to recompute.
                return None;
            }
        }
        // Filter latest time signatures in the time window.
        let lower_bound = curr_ts.as_nanos() - self.time_window.as_nanos();
        let last_sigs: Vec<&Chronicle> = pool
            .as_slice()
            .into_iter()
            .filter(|el| el.epoch > lower_bound.into() && el.epoch <= pool.last().unwrap().epoch)
            .collect();
        // Final mean time computation.
        let sum_time: u128 = last_sigs
            .as_slice()
            .into_iter()
            .map(|el| el.epoch.as_u128())
            .sum();
        let mean_time = sum_time / last_sigs.len() as u128;
        let last_sigs = last_sigs.into_iter().map(|el| el.clone()).collect();
        // Cleanup the pool.
        pool.clear();
        return Some((mean_time.into(), last_sigs));
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc, time::Duration};

    use ethers::{
        providers::{MockProvider, Provider},
        types::{Address, Bytes},
    };
    use tokio::sync::Mutex;

    use crate::time_signature::Chronicle;

    use super::MeanTime;

    #[tokio::test]
    async fn test_compute_mean_time() -> Result<(), String> {
        let pool_vec = vec![
            Chronicle::new(
                Duration::new(1734220767, 0).as_nanos().into(),
                Address::from_str("0x25ee756f5d93e26f5011b7ed4866afb192ce483e").unwrap(),
                Bytes::from_str("0x72315c2259bd482317373295b6f3985e889fcdea6b50ef7344e89a417f7bf6645aac1039674909c314e02be38dc377997a8ea682b366fe1af9a4eb919815140f1c").unwrap()
            ),
            Chronicle::new(
                Duration::new(1734220768, 0).as_nanos().into(),
                Address::from_str("0x25ee756f5d93e26f5011b7ed4866afb192ce483e").unwrap(),
                Bytes::from_str("0x72315c2259bd482317373295b6f3985e889fcdea6b50ef7344e89a417f7bf6645aac1039674909c314e02be38dc377997a8ea682b366fe1af9a4eb919815140f1c").unwrap()
            ),
            Chronicle::new(
                Duration::new(1734220760, 0).as_nanos().into(),
                Address::from_str("0x25ee756f5d93e26f5011b7ed4866afb192ce483e").unwrap(),
                Bytes::from_str("0x72315c2259bd482317373295b6f3985e889fcdea6b50ef7344e89a417f7bf6645aac1039674909c314e02be38dc377997a8ea682b366fe1af9a4eb919815140f1c").unwrap()
            ),
        ];
        let time_window = parse_duration::parse("2s").unwrap();
        let pool = Arc::new(Mutex::new(pool_vec));
        let mean_time = MeanTime::new(
            pool.clone(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            time_window,
        );
        let test_res_opt = mean_time.compute_mean_time(Duration::new(1734220768, 0)).await;
        assert_ne!(test_res_opt, None);
        let (mean_time, sigs) = test_res_opt.unwrap();
        assert_eq!(
            mean_time,
            Duration::new(1734220767, 500000000).as_nanos().into()
        );
        assert_eq!(sigs.len(), 2);
        Ok(())
    }
}
