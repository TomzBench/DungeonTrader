/// plugin
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Input {
    Airdrop,
    Buy,
    Donate,
    Gift,
    Hardfork,
    Income,
    Interest,
    Mining,
    Staking,
    Wages,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Output {
    Donate,
    Fee,
    Gift,
    Lost,
    Sell,
    Staking,
}

#[derive(Serialize, Deserialize)]
pub struct InputTransaction {
    pub timestamp: u64,
    pub asset: String,
    pub exchange: String,
    pub holder: String,
    #[serde(rename = "type")]
    pub typ: Input,
    pub spot_price: f32,
    pub crypto_in: f32,
    pub crypto_fee: f32,
    pub fiat_in_no_fee: f32,
    pub fiat_in_with_fee: f32,
    pub fiat_fee: f32,
    pub unique_id: u32,
    pub notes: String,
}

#[derive(Serialize, Deserialize)]
pub struct OutputTransaction {
    pub timestamp: f32,
    pub asset: String,
    pub exchange: String,
    pub holder: String,
    pub typ: Output,
    pub spot_price: f32,
    pub crypto_out_no_fee: f32,
    pub crypto_fee: f32,
    pub crypto_out_with_fee: f32,
    pub fiat_out_no_fee: f32,
    pub fiat_fee: f32,
    pub unique_id: u32,
    pub notes: String,
}

#[derive(Serialize, Deserialize)]
pub struct IntraTransaction {
    pub timestamp: f32,
    pub asset: String,
    pub from_exchange: String,
    pub from_holder: String,
    pub to_exchange: String,
    pub to_holder: String,
    pub spot_price: f32,
    pub crypto_sent: f32,
    pub crypto_received: f32,
    pub unique_id: u32,
    pub notes: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transaction {
    Input(InputTransaction),
    Output(OutputTransaction),
    Intra(IntraTransaction),
}
