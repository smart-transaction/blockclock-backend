use ethers::types::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    pub time_keeper: Address,
    pub avatar: String,
}
