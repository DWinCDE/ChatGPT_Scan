use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;
use async_trait::async_trait;
use base::params::UserData;
use crate::state::UserState;

#[async_trait]
pub trait ExchangeUserClient {
    async fn start_user_order(&self, shared_state: Arc<RwLock<UserState>>);
    async fn start_user_balance(&self, shared_state: Arc<RwLock<UserState>>);
}

pub trait ExchangeUserCertificate {
    fn signature(&self, api_key: Option<String>, secret_key: Option<String>, channel: UserData) -> Value;
}
// [#2 Order] Filled SUCCESS: Order { symbol: "btctwd", order_id: "8807813106", client_id: "null", label: "-", side: SELL, order_type: LIMIT, time_in_force: GTC, price: "1980000.0", amount: "0.001", status: FILLED, filled_price: "2016100.0", filled_amount: "0.001", remaining_amount: "0.0", created_ts: 1719548360000, updated_ts: 1719548360033 }
// let third_order_amount: String = (second_order_result.filled_amount.parse::<f64>().unwrap_or(0.0) * second_order_result.filled_price.parse::<f64>().unwrap_or(0.0) / arbitrage_opportunity.booktickers[0].ask_price).to_string();

// 0.001 * 2016100.0