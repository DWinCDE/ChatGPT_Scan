// pub mod binance;
pub mod maicoin;

use serde_json::Value;
use base::params::{Symbol, OrderSide, OrderType};
use base::models::{Order};
use base::errors::EnumError;
use crate::models::{Ticker, Orderbook};

#[async_trait::async_trait]
pub trait Exchange {
    async fn get_exchange_info(&self) -> Result<Value, EnumError>;
    async fn get_ticker(&self, symbol: Symbol) -> Result<Ticker, EnumError>; 
    async fn get_orderbook(&self, symbol: Symbol) -> Result<Orderbook, EnumError>; 
    async fn get_account(&self) -> Result<serde_json::Value, EnumError>;
    async fn get_open_orders(&self, symbol: Symbol) -> Result<serde_json::Value, EnumError>;
    async fn create_order(&self, new_order: Order) -> Result<Order, EnumError>; 
    async fn cancel_order(&self, symbol: Symbol, order_id: &str) -> Result<serde_json::Value, EnumError>;
}