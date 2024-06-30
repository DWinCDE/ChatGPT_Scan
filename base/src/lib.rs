pub mod utils;
pub mod params;
pub mod models;
pub mod errors;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api_info: ApiConfig,
    pub settings: SettingsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub account_name: String,
    pub exchange: String,
    pub api_key: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SettingsConfig {
    pub fee_rate: f64,
    pub response_timeout: u64,
    pub protect_tolerance: f64,
}