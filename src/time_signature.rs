use ethers::{
    prelude::abigen,
    types::{Address, Bytes, Signature, U256},
};
use log::error;

abigen!(
  BlockTime,
  "./abi/BlockTime.sol/BlockTime.json",
  derives(serde::Deserialize, serde::Serialize);
);

impl Chronicle {
    pub fn new(epoch: U256, time_keeper: Address, signature: Bytes) -> Chronicle {
        Chronicle {
            epoch,
            time_keeper,
            signature,
        }
    }

    pub fn verify(&self) -> bool {
        // TODO: Make sure what message is signed.
        match Signature::try_from(self.signature.to_vec().as_slice()) {
            Ok(signature) => {
                if let Err(err) = signature.verify(self.epoch.to_string(), self.time_keeper) {
                    error!("Error signature verification: {}", err);
                    return false;
                }
                return true;
            }
            Err(err) => {
                error!("Error parsing signature: {}", err);
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chronicle;
    use ethers::types::{Address, Bytes, U256};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_verify() -> Result<(), String> {
        let time_keeper = Address::from_str("0x2c57d1CFC6d5f8E4182a56b4cf75421472eBAEa4").unwrap();
        let time_sig = Chronicle::new(
            U256::from_dec_str("1734554316445000000").unwrap(),
            time_keeper,
            Bytes::from_str("0x99d6d06c0e655a617cb043aed547410d7575466ffe36f907d410b03ea7e63e2456ddeace270811317fc1360678f682124944e76484e1019d7c1f5b8cdfb91c131b").unwrap()
        );
        assert!(time_sig.verify());
        Ok(())
    }
}
