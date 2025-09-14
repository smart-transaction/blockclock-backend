use crate::time_signature::BlockTime;
use ethers::{
    abi::{encode, Token},
    prelude::abigen,
    providers::Middleware,
    signers::LocalWallet,
    types::{Address, Bytes, Signature, H256, U256},
    utils::{hash_message, keccak256},
};
use std::sync::Arc;

abigen!(
  CallBreaker,
  "abi/CallBreaker.sol/CallBreaker.json",
  derives(serde::Deserialize, serde::Serialize);
);

pub struct CallBreakerData<M: Middleware> {
    pub call_breaker_contract: CallBreaker<M>,
    pub block_time_contract: BlockTime<M>,
    pub solver_wallet: LocalWallet,
    pub validator_wallet: LocalWallet,
    pub app_id: Bytes,
}

impl<M: Middleware> CallBreakerData<M> {
    pub fn new(
        call_breaker_address: Address,
        block_time_address: Address,
        client: Arc<M>,
        solver_private_key: LocalWallet,
        validator_private_key: LocalWallet,
        app_id: Bytes,
    ) -> CallBreakerData<M> {
        CallBreakerData {
            call_breaker_contract: CallBreaker::new(call_breaker_address, client.clone()),
            block_time_contract: BlockTime::new(block_time_address, client.clone()),
            solver_wallet: solver_private_key,
            validator_wallet: validator_private_key,
            app_id,
        }
    }
}

impl CallObject {
    pub fn new(
        salt: U256,
        amount: U256,
        gas: U256,
        addr: Address,
        callvalue: Bytes,
        returnvalue: Bytes,
        skippable: bool,
        verifiable: bool,
        expose_return: bool,
    ) -> CallObject {
        CallObject {
            salt,
            amount,
            gas,
            addr,
            callvalue,
            returnvalue,
            skippable,
            verifiable,
            expose_return,
        }
    }

    pub fn to_token_tuple(&self) -> Token {
        Token::Tuple(vec![
            Token::Uint(self.salt),
            Token::Uint(self.amount),
            Token::Uint(self.gas),
            Token::Address(self.addr),
            Token::Bytes(self.callvalue.clone().to_vec()),
            Token::Bytes(self.returnvalue.clone().to_vec()),
            Token::Bool(self.skippable),
            Token::Bool(self.verifiable),
            Token::Bool(self.expose_return),
        ])
    }
}

impl UserObjective {
    pub fn new(
        app_id: Bytes,
        nonce: U256,
        tip: U256,
        chain_id: U256,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: U256,
        sender: Address,
        signer_private_key: LocalWallet,
        call_objects: Vec<CallObject>,
    ) -> UserObjective {
        UserObjective {
            app_id,
            nonce,
            tip,
            chain_id,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            sender,
            signature: Self::sender_signature(&nonce, &sender, &signer_private_key, &call_objects),
            call_objects,
        }
    }

    fn sender_signature(
        nonce: &U256,
        sender: &Address,
        signer_private_key: &LocalWallet,
        call_objects: &Vec<CallObject>,
    ) -> Bytes {
        // generate the message hash
        let call_tokens: Vec<Token> = call_objects.iter().map(|c| c.to_token_tuple()).collect();
        let encoded_callobjects = encode(&[Token::Array(call_tokens)]);

        let data = encode(&[
            Token::Uint(*nonce),
            Token::Address(*sender),
            Token::Bytes(encoded_callobjects),
        ]);
        let hash_bytes = keccak256(&data);
        let hash = H256::from_slice(&hash_bytes); // convert [u8; 32] → H256

        // Ethereum-specific message prefix (EIP-191)
        let eth_hash = hash_message(hash);

        let sig: Signature = signer_private_key.sign_hash(eth_hash).unwrap();

        // Convert into 65-byte compact form
        let compact: [u8; 65] = sig.to_vec().try_into().unwrap();

        Bytes::from(compact.to_vec())
    }
}

impl AdditionalData {
    pub fn new(key: H256, value: Bytes) -> AdditionalData {
        AdditionalData {
            key: key.into(),
            value,
        }
    }

    pub fn to_token_tuple(&self) -> Token {
        Token::Tuple(vec![
            Token::FixedBytes(self.key.to_vec()),
            Token::Bytes(self.value.clone().to_vec()),
        ])
    }
}

impl MevTimeData {
    pub fn new(
        validator_private_key: LocalWallet,
        mev_time_data_values: Vec<AdditionalData>,
    ) -> MevTimeData {
        MevTimeData {
            validator_signature: Self::validator_signature(
                &mev_time_data_values,
                &validator_private_key,
            ),
            mev_time_data_values,
        }
    }

    fn validator_signature(
        data: &Vec<AdditionalData>,
        validator_private_key: &LocalWallet,
    ) -> Bytes {
        // generate the message hash
        let additional_data_token: Vec<Token> = data.iter().map(|c| c.to_token_tuple()).collect();
        let additional_data_encoded = encode(&[Token::Array(additional_data_token)]);

        let hash_bytes = keccak256(&additional_data_encoded);
        let hash = H256::from_slice(&hash_bytes); // convert [u8; 32] → H256

        // Ethereum-specific message prefix (EIP-191)
        let eth_hash = hash_message(hash);

        let sig: Signature = validator_private_key.sign_hash(eth_hash).unwrap();

        // Convert into 65-byte compact form
        let compact: [u8; 65] = sig.to_vec().try_into().unwrap();

        Bytes::from(compact.to_vec())
    }
}
