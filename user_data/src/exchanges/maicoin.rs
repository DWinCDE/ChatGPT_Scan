use chrono::Utc;
use hex::encode;
use sha2::Sha256;
use std::sync::Arc;
use hmac::{Hmac, Mac};
use std::time::Instant;
use tokio::sync::RwLock;
use async_trait::async_trait;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::state::UserState;
use crate::models::{OrderMessageUpdate, BalanceMessageUpdate};
use crate::ws_client::{ExchangeUserClient, ExchangeUserCertificate};
use base::errors::EnumError;
use base::utils::{symbol_to_enum, convert_str_to_decimal};
use base::models::{Order, CurencyBalance};
use base::params::{ExchangeParams, OrderSide, OrderStatus, OrderType, Symbol, TimeInForce, UserData};
use websocket_client::WebSocketClient;
type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct MaiCoinUserWsClient {
    api_key: Option<String>,
    secret_key: Option<String>
}

impl MaiCoinUserWsClient {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        MaiCoinUserWsClient {
            api_key,
            secret_key
        }
    }
}

impl ExchangeUserCertificate for MaiCoinUserWsClient {
    fn signature(&self, api_key: Option<String>, secret_key: Option<String>, channel: UserData) -> Value {
        let channel_message = match channel {
            UserData::ACCOUNT_ORDERS => "order",
            UserData::ACCOUNT_BALANCE => "account",
            _ => ""
        };
        let ts = Utc::now().timestamp_millis();
        let ts_message = ts.to_string();
        let secret_key = match secret_key {
            Some(key) => key,
            None => panic!("Secret key is required for signing"),
        };
        let mut mac = HmacSha256::new_from_slice(&secret_key.as_bytes()).unwrap();        
        mac.update(ts_message.as_bytes());
        let signature = encode(mac.finalize().into_bytes());

        json!({
                "action": "auth",
                "apiKey": &api_key,
                "nonce": ts,
                "signature": signature,
                "filters": [channel_message]
        })
    }
}

#[async_trait]
impl ExchangeUserClient for MaiCoinUserWsClient {
    async fn start_user_order(&self, shared_state: Arc<RwLock<UserState>>) {
        let subscribe_message = self.signature(self.api_key.clone(), self.secret_key.clone(), UserData::ACCOUNT_ORDERS);
        let url = "wss://max-stream.maicoin.com/ws";
        let shared_state = shared_state.clone();
        let ws_client = WebSocketClient::new(&url, Some(subscribe_message.to_string()));

        tokio::spawn(async move {
            ws_client.start(move |msg| {
                let shared_state = shared_state.clone();
                tokio::spawn(async move {
                    if let Ok(order_update_message) = serde_json::from_str::<MaiCoinOrderMessage>(&msg) {
                        match order_update_message.event.as_str() {
                            "order_update" | "order_snapshot" => {
                                let mut state = shared_state.write().await;
                                for order_message in order_update_message.orders.iter() {
                                    let order = order_update_message.order_update(&order_message);
                                    state.account_orders.insert(order.order_id.clone(), order);
                                }
                            }
                            _ => {
                                println!("Unhandled event: {}", order_update_message.event);
                            }
                        }
                    }
                });
            }).await;
        });
    }

    async fn start_user_balance(&self, shared_state: Arc<RwLock<UserState>>) {
        let subscribe_message = self.signature(self.api_key.clone(), self.secret_key.clone(), UserData::ACCOUNT_BALANCE);
        let url = "wss://max-stream.maicoin.com/ws";
        let shared_state = shared_state.clone();
        let ws_client = WebSocketClient::new(&url, Some(subscribe_message.to_string()));

        tokio::spawn(async move {
            ws_client.start(move |msg| {
                let shared_state = shared_state.clone();
                tokio::spawn(async move {
                    if let Ok(balance_update_message) = serde_json::from_str::<MaiCoinBalanceMessage>(&msg) {
                        match balance_update_message.event.as_str() {
                            "account_update" | "account_snapshot" => {
                                let mut state = shared_state.write().await;
                                let balance_update_message_clone = balance_update_message.clone();
                                for balance_message in balance_update_message.balances {
                                    let balance = balance_update_message_clone.balance_update(&balance_message);
                                    state.account_balances.insert(balance.currency.clone(), balance);
                                }
                            }
                            _ => {
                                println!("Unhandled event: {}", balance_update_message.event);
                            }
                        }
                    }
                });
            }).await;
        });
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MaiCoinAuthMessage {
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "T")]
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MaiCoinOrderMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "o")]
    pub orders: Vec<serde_json::Value>,
    #[serde(rename = "T")]
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MaiCoinBalanceMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "B")]
    pub balances: Vec<serde_json::Value>,
    #[serde(rename = "T")]
    pub timestamp: u64,
}

impl OrderMessageUpdate for MaiCoinOrderMessage {
    fn order_update(&self, order_message: &Value) -> Order {
        let order_side = match order_message.get("sd") {
            Some(side_value) => {
                match side_value.as_str() {
                    Some("bid") => OrderSide::BUY,
                    Some("ask") => OrderSide::SELL,
                    _ => OrderSide::UNKNOWN_ORDER_SIDE
                }
            }
            None => OrderSide::UNKNOWN_ORDER_SIDE
        };

        let order_type = match order_message.get("ot") {
            Some(order_type_value) => {
                match order_type_value.as_str() {
                    Some("market") => OrderType::MARKET,
                    Some("stop_market") => OrderType::MARKET,
                    Some("post_only") => OrderType::LIMIT,
                    Some("limit") => OrderType::LIMIT,
                    Some("ioc_limit") => OrderType::LIMIT,
                    Some("stop_limit") => OrderType::LIMIT,
                    _ => OrderType::UNKNOWN_ORDER_TYPE
                }
            }
            None => OrderType::UNKNOWN_ORDER_TYPE
        };

        let time_in_force = match order_message.get("ot") {
            Some(time_in_force_value) => {
                match time_in_force_value.as_str() {
                    Some("market") => TimeInForce::GTC,
                    Some("stop_market") => TimeInForce::GTC,
                    Some("post_only") => TimeInForce::MAKER_ONLY,
                    Some("limit") => TimeInForce::GTC,
                    Some("ioc_limit") => TimeInForce::IOC,
                    Some("stop_limit") => TimeInForce::GTC,
                    _ => TimeInForce::UNKNOWN_TIMEINFORCE
                }
            }
            None => TimeInForce::UNKNOWN_TIMEINFORCE
        };

        let order_status: OrderStatus = match order_message.get("S") {
            Some(order_status_value) => {
                match order_status_value.as_str() {
                    Some("wait") => OrderStatus::NEW,
                    Some("cancel") => OrderStatus::CANCEL,
                    Some("done") => OrderStatus::FILLED,
                    _ => OrderStatus::UNKNOWN_STATUS
                }
            }
            None => OrderStatus::UNKNOWN_STATUS
        };
    
        let price = match order_type {
            OrderType::MARKET => convert_str_to_decimal(order_message.get("ap").unwrap().as_str().unwrap()),
            _ => convert_str_to_decimal(order_message.get("ap").unwrap().as_str().unwrap())
        };

        // println!("{}", order_message);
        Order {
            symbol: symbol_to_enum(&order_message.get("M").unwrap().as_str().unwrap().to_string()),
            order_id: order_message.get("i").unwrap().to_string(),
            client_id: order_message.get("ci").unwrap().to_string(),
            label: "-".to_string(),
            side: order_side,
            order_type: order_type,
            time_in_force: time_in_force,
            price: price,
            amount: convert_str_to_decimal(order_message.get("v").unwrap().as_str().unwrap()),
            status: order_status,
            filled_price: convert_str_to_decimal(order_message.get("ap").unwrap().as_str().unwrap()),
            filled_amount: convert_str_to_decimal(order_message.get("ev").unwrap().as_str().unwrap()),
            remaining_amount: convert_str_to_decimal(order_message.get("rv").unwrap().as_str().unwrap()),
            created_ts: order_message.get("T").unwrap().as_u64().unwrap(),
            updated_ts: order_message.get("TU").unwrap().as_u64().unwrap(),
        }
        
    }
}

impl BalanceMessageUpdate for MaiCoinBalanceMessage {
    fn balance_update(&self, balance_message: &Value) -> CurencyBalance {
        let available = match balance_message["av"].as_str() {
            Some("null") | None => "0".to_string(),
            Some(availableValue) => availableValue.to_string(),
            _ => "0".to_string(),
        };

        let locked = match balance_message["l"].as_str() {
            Some("null") | None => "0".to_string(),
            Some(lockedValue) => lockedValue.to_string(),
            _ => "0".to_string(),
        };

        let staked = match balance_message["stk"].as_str() {
            Some("null") | None => "0".to_string(),
            Some(stakedValue) => stakedValue.to_string(),
            _ => "0".to_string(),
        };

        CurencyBalance {
            currency: balance_message.get("cu").unwrap().as_str().unwrap().to_string().to_uppercase(),
            available: available,
            locked: locked,
            staked: staked,
            updated_ts: balance_message.get("TU").unwrap().as_u64().unwrap(),
            
        }
    }

}