use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use ethers::{
    abi::{encode, Token},
    providers::Middleware,
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, U256},
    utils::{keccak256, parse_units},
};

use log::{error, info};
use md5::{Context, Digest};
use mysql::PooledConn;
use tokio::{spawn, sync::Mutex};

use crate::{
    address_str::get_address_strings,
    call_breaker::{AdditionalData, CallBreakerData, CallObject, MevTimeData, UserObjective},
    referrers_fetch::read_referrers_list,
    time_pool::TimeSigPool,
    time_signature::Chronicle,
};

pub struct MeanTime<M: Middleware> {
    pool: Arc<Mutex<TimeSigPool>>,
    primary_call_breaker_comp: Arc<CallBreakerData<M>>,
    secondary_call_breaker_comp: Arc<CallBreakerData<M>>,
    time_window: Duration,
    curr_md5: Digest,
    is_dry_run: bool,
}

const TIME_KEEPER_REWARD: f64 = 1.0;
static NONCE: AtomicU32 = AtomicU32::new(0);

async fn send_rewards<M: Middleware>(
    last_sigs: Vec<Chronicle>,
    mean_time: U256,
    all_receivers: Vec<Address>,
    all_amounts: Vec<U256>,
    call_breaker_data: Arc<CallBreakerData<M>>,
) -> bool {
    // generate user_objective
    let user_objective: UserObjective = prepare_call_and_user_objective(
        &last_sigs,
        &mean_time,
        &all_receivers,
        &all_amounts,
        &call_breaker_data,
    );

    let user_objectives = vec![user_objective];
    let returns_bytes = vec![Bytes::new()];
    let order_of_execution = vec![U256::from(0)];

    // generate mev_time_data
    let mev_time_data = prepare_mev_time_data(
        &last_sigs,
        &mean_time,
        &all_receivers,
        &all_amounts,
        &call_breaker_data.validator_wallet,
    );

    let estimated_gas = call_breaker_data
        .call_breaker_contract
        .execute_and_verify(
            user_objectives.clone(),
            returns_bytes.clone(),
            order_of_execution.clone(),
            mev_time_data.clone(),
        )
        .estimate_gas()
        .await;

    let gas_limit = estimated_gas.unwrap() * 120 / 100;
    match call_breaker_data
        .call_breaker_contract
        .execute_and_verify(
            user_objectives,
            returns_bytes,
            order_of_execution,
            mev_time_data,
        )
        .gas(gas_limit)
        .send()
        .await
    {
        Ok(pending) => {
            info!("Transaction is sent, txhash: {}", pending.tx_hash());
            match pending.await {
                Ok(receipt) => {
                    if let Some(receipt) = receipt {
                        if let Some(status) = receipt.status {
                            info!("Got transaction status: {}", status);
                            NONCE.fetch_add(1, Ordering::SeqCst);
                            return true;
                        }
                    }
                    error!("Transaction status wasn't received.");
                    return false;
                }
                Err(err) => {
                    error!("Error pending transaction: {}", err);
                    return false;
                }
            }
        }
        Err(err) => {
            error!("Error sending transaction: {}", err);
            return false;
        }
    }
}

fn prepare_call_and_user_objective<M: Middleware>(
    last_sigs: &[Chronicle],
    mean_time: &U256,
    all_receivers: &[Address],
    all_amounts: &[U256],
    call_breaker_data: &Arc<CallBreakerData<M>>,
) -> UserObjective {
    let call = call_breaker_data
        .block_time_contract
        .method::<(Vec<Chronicle>, U256, Vec<Address>, Vec<U256>), ()>(
            "moveTime",
            (
                last_sigs.to_vec(),
                *mean_time,
                all_receivers.to_vec(),
                all_amounts.to_vec(),
            ),
        )
        .unwrap();
    let calldata = call.calldata().unwrap();

    let call_object = CallObject::new(
        U256::from(1),
        U256::from(0),
        U256::from(1_000_000),
        call_breaker_data.block_time_contract.address(),
        calldata,
        Bytes::new(),
        true,
        false,
        true,
    );

    UserObjective::new(
        call_breaker_data.app_id.clone(),
        U256::from(NONCE.load(Ordering::SeqCst)),
        U256::from(0),
        U256::from(1),
        U256::from(0),
        U256::from(0),
        Address::from(call_breaker_data.solver_wallet.address()),
        call_breaker_data.solver_wallet.clone(),
        vec![call_object],
    )
}

fn prepare_mev_time_data(
    last_sigs: &[Chronicle],
    mean_time: &U256,
    all_receivers: &[Address],
    all_amounts: &[U256],
    validator_wallet: &LocalWallet,
) -> MevTimeData {
    let last_sig_token: Vec<Token> = last_sigs.iter().map(|c| c.to_token_tuple()).collect();
    let last_sig_encoded = encode(&[Token::Array(last_sig_token)]);
    let last_sig_bytes = Bytes::from(last_sig_encoded);

    let mean_time_encoded = encode(&[Token::Uint(*mean_time)]);
    let mean_time_bytes = Bytes::from(mean_time_encoded);

    let all_receivers_tokens: Vec<Token> =
        all_receivers.iter().copied().map(Token::Address).collect();
    let all_receivers_encoded = encode(&[Token::Array(all_receivers_tokens)]);
    let all_receivers_bytes = Bytes::from(all_receivers_encoded);

    let all_amounts_tokens: Vec<Token> = all_amounts.iter().copied().map(Token::Uint).collect();
    let all_amounts_encoded = encode(&all_amounts_tokens);
    let all_amounts_bytes = Bytes::from(all_amounts_encoded);

    let mev_time_data_values = vec![
        AdditionalData::new(keccak256(b"Chronicles").into(), last_sig_bytes),
        AdditionalData::new(keccak256(b"CurrentMeanTime").into(), mean_time_bytes),
        AdditionalData::new(keccak256(b"Receivers").into(), all_receivers_bytes),
        AdditionalData::new(keccak256(b"Amounts").into(), all_amounts_bytes),
    ];

    MevTimeData::new(validator_wallet.clone(), mev_time_data_values)
}

impl<M: Middleware + 'static> MeanTime<M> {
    pub fn new(
        pool: Arc<Mutex<TimeSigPool>>,
        primary_call_breaker_comp: Arc<CallBreakerData<M>>,
        secondary_call_breaker_comp: Arc<CallBreakerData<M>>,
        time_window: Duration,
        is_dry_run: bool,
    ) -> MeanTime<M> {
        MeanTime {
            pool,
            primary_call_breaker_comp,
            secondary_call_breaker_comp,
            time_window,
            curr_md5: md5::compute("--dummy--"),
            is_dry_run,
        }
    }

    async fn compute_mean_time(&self, curr_ts: Duration) -> Option<(U256, Vec<Chronicle>)> {
        // Check the latest signature.
        let mut pool = self.pool.lock().await;
        if pool.is_empty() {
            return None;
        }
        pool.sort_by_key(|el| el.epoch);
        // Filter latest time signatures in the time window.
        let upper_bound: U256;
        if pool.last().unwrap().epoch > curr_ts.as_nanos().into() {
            // The last time is newer than the current server time, considering server time
            upper_bound = curr_ts.as_nanos().into();
        } else {
            // The last time is earlier than the current server time, considering the last time
            upper_bound = pool.last().unwrap().epoch;
        }
        let lower_bound = upper_bound - self.time_window.as_nanos();
        let last_sigs: Vec<&Chronicle> = pool
            .as_slice()
            .into_iter()
            .filter(|el| el.epoch > lower_bound.into() && el.epoch <= upper_bound)
            .collect();
        // Final mean time computation.
        let sum_time: u128 = last_sigs
            .as_slice()
            .into_iter()
            .map(|el| el.epoch.as_u128())
            .sum();
        if last_sigs.is_empty() {
            return None;
        }
        let mean_time = sum_time / last_sigs.len() as u128;
        let last_sigs = last_sigs.into_iter().map(|el| el.clone()).collect();
        pool.clear();
        return Some((mean_time.into(), last_sigs));
    }

    pub async fn handle_time_tick(&mut self, curr_ts: SystemTime, conn: Arc<Mutex<PooledConn>>) {
        // Get mean time
        let curr_ts_epoch = curr_ts.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        if let Some((mean_time, last_sigs)) = self.compute_mean_time(curr_ts_epoch).await {
            let curr_md5_ctx =
                last_sigs
                    .as_slice()
                    .into_iter()
                    .fold(Context::new(), |mut acc, el| {
                        acc.consume(&el.signature);
                        return acc;
                    });
            let curr_md5 = curr_md5_ctx.compute();
            if curr_md5 == self.curr_md5 {
                // No changes, no need to update the time.
                return;
            }
            // Send the mean time and signatures to the contract
            let mut accounts_and_amounts =
                last_sigs
                    .as_slice()
                    .iter()
                    .fold(BTreeMap::new(), |mut acc, el| {
                        let (account, _) = get_address_strings(&el.time_keeper);
                        match acc.get(&account) {
                            Some(amount) => {
                                acc.insert(account, amount + TIME_KEEPER_REWARD);
                            }
                            None => {
                                acc.insert(account, TIME_KEEPER_REWARD);
                            }
                        }
                        acc
                    });

            {
                let mut conn = conn.lock().await;
                if let Err(err) =
                    read_referrers_list(conn.as_mut(), &mut accounts_and_amounts).await
                {
                    error!("Error getting referrers: {}", err);
                    return;
                }
            }
            let (all_receivers, all_amounts) = accounts_and_amounts.into_iter().fold(
                (Vec::new(), Vec::new()),
                |mut acc: (Vec<Address>, Vec<U256>), el| {
                    if let Ok(account) = el.0.parse::<Address>() {
                        acc.0.push(account);
                        if let Ok(amount) = parse_units(el.1, "ether") {
                            acc.1.push(amount.into());
                        }
                    }
                    acc
                },
            );
            // Added for suspending rewards during airdrop.
            if self.is_dry_run {
                info!(
                    "Skipping sending rewards due to dry_run mode, skipped rewards:\n{:#?} {:#?}",
                    all_receivers, all_amounts
                );
                return;
            }
            let primary_last_sigs = last_sigs.clone();
            let primary_all_receivers = all_receivers.clone();
            let primary_all_amounts = all_amounts.clone();
            let primary_call_breaker_comp = self.primary_call_breaker_comp.clone();
            let primary_handle = spawn(async move {
                return send_rewards(
                    primary_last_sigs,
                    mean_time,
                    primary_all_receivers,
                    primary_all_amounts,
                    primary_call_breaker_comp,
                )
                .await;
            });

            let secondary_call_breaker_comp = self.secondary_call_breaker_comp.clone();
            let secondary_handle = spawn(async move {
                return send_rewards(
                    last_sigs,
                    mean_time,
                    all_receivers,
                    all_amounts,
                    secondary_call_breaker_comp,
                )
                .await;
            });
            match primary_handle.await {
                Ok(success) => {
                    if success {
                        self.curr_md5 = curr_md5;
                    }
                }
                Err(err) => error!("Error executing the primary awards disbursement: {}", err),
            }
            if let Err(err) = secondary_handle.await {
                error!("Error executing the secondary awards disbursement: {}", err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc, time::Duration};

    use ethers::{
        providers::{MockProvider, Provider},
        signers::LocalWallet,
        types::{Address, Bytes},
    };
    use tokio::sync::Mutex;

    use crate::{call_breaker::CallBreakerData, time_signature::Chronicle};

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
        let primary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let secondary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let mean_time = MeanTime::new(
            pool.clone(),
            primary_call_breaker_comp,
            secondary_call_breaker_comp,
            time_window,
            false,
        );
        let test_res_opt = mean_time
            .compute_mean_time(Duration::new(1734220768, 0))
            .await;
        assert_ne!(test_res_opt, None);
        let (mean_time_val, sigs) = test_res_opt.unwrap();
        assert_eq!(
            mean_time_val,
            Duration::new(1734220767, 500000000).as_nanos().into()
        );
        assert_eq!(sigs.len(), 2);
        let remaining_pool = mean_time.pool.lock().await;
        assert!(remaining_pool.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_mean_time_last_too_new() -> Result<(), String> {
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
        let primary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let secondary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let mean_time = MeanTime::new(
            pool.clone(),
            primary_call_breaker_comp,
            secondary_call_breaker_comp,
            time_window,
            false,
        );
        let test_res_opt = mean_time
            .compute_mean_time(Duration::new(1734220767, 0))
            .await;
        assert_ne!(test_res_opt, None);
        let (mean_time_val, sigs) = test_res_opt.unwrap();
        assert_eq!(
            mean_time_val,
            Duration::new(1734220767, 0).as_nanos().into()
        );
        assert_eq!(sigs.len(), 1);
        let remaining_pool = mean_time.pool.lock().await;
        assert!(remaining_pool.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_compute_mean_time_empty() -> Result<(), String> {
        let pool_vec = Vec::new();
        let time_window = parse_duration::parse("2s").unwrap();
        let pool = Arc::new(Mutex::new(pool_vec));
        let primary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let secondary_call_breaker_comp = Arc::new(CallBreakerData::new(
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a022").unwrap(),
            Address::from_str("0x8ab3c48c839376d2b79ab98f23f5b2406a06a029").unwrap(),
            Arc::new(Provider::new(MockProvider::new())),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            LocalWallet::from_str(
                "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
            Bytes::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
        ));
        let mean_time = MeanTime::new(
            pool.clone(),
            primary_call_breaker_comp,
            secondary_call_breaker_comp,
            time_window,
            false,
        );
        let test_res_opt = mean_time
            .compute_mean_time(Duration::new(1734220768, 0))
            .await;
        assert_eq!(test_res_opt, None);
        Ok(())
    }
}
