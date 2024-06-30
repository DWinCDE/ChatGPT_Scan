use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use base::params::{Symbol, OrderSide, OrderType, OrderStatus, TimeInForce};
use base::models::{Order};

// [ Public ] Market Data Struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    pub bid: String,
    pub ask: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Orderbook {
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
    pub timestamp: u64,
    pub update_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderbookEntry {
    pub price: String,
    pub quantity: String,
}

// [ Private ] Trade Data Struct
// =======================================================================================================================================
#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeResponse {
    pub ticker: Option<Ticker>,
    pub orderbook: Option<Orderbook>,
}

pub trait ExchangeResponseMapper {
    fn safe_ticker(&self, response: &serde_json::Value) -> Ticker;
    fn safe_orderbook(&self, response: &serde_json::Value) -> Orderbook;
    fn safe_order(&self, response: &serde_json::Value) -> Order;
}