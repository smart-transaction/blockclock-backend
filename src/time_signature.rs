use ethers::{
    prelude::abigen,
    types::{Address, Bytes, Signature, U256},
};

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
                    println!("Error signature verification: {}", err);
                    return false;
                }
                return true;
            }
            Err(err) => {
                println!("Error parsing signature: {}", err);
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

    #[test]
    fn test_verify() -> Result<(), String> {
        let time_keeper = Bytes::from_str("0x807e381C344AcC6af14A75c5E1b8C82a92dE3F68").unwrap();
        let time_sig = Chronicle::new(
            U256::from_dec_str("1734554316445000000").unwrap(),
            Address::from_slice(time_keeper.to_vec().as_slice()),
            Bytes::from_str("0xe843cb59fd2f060cbdb887f7b376309387771fee9104468511180742f25b35520ffeea8199087571b601f1c83bb37a5509811482f13d33f37329e2ca8ba728e61c").unwrap()
        );
        assert!(time_sig.verify());
        Ok(())
    }
}
