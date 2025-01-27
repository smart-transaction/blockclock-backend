use ethers::types::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    pub time_keeper: Address,
    pub avatar: String,
    pub referral_code: String,
    pub referred_from: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvatarData {
    pub time_keeper: Address,
    pub avatar: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferralCodeData {
    pub time_keeper: Address,
    pub referral_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferredFromData {
    pub time_keeper: Address,
    pub referred_from: String,
}
