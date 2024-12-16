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
    use ethers::types::{Address, Bytes};
    use std::str::FromStr;

    #[test]
    fn test_verify() -> Result<(), String> {
        let time_keeper = Bytes::from_str("0x25ee756f5d93e26f5011b7ed4866afb192ce483e").unwrap();
        let time_sig = Chronicle::new(
            1234567890.into(),
            Address::from_slice(time_keeper.to_vec().as_slice()),
            Bytes::from_str("0x72315c2259bd482317373295b6f3985e889fcdea6b50ef7344e89a417f7bf6645aac1039674909c314e02be38dc377997a8ea682b366fe1af9a4eb919815140f1c").unwrap()
        );
        assert!(time_sig.verify());
        Ok(())
    }
}
