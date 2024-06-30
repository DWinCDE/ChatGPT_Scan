use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use base::{errors::EnumError, models::Order};
use base::params::Symbol;
use base::errors::TradeError;
use serde_json::Value;
use user_data::state::{UserState, UserStateHandle};
use strategy::ArbitrageOpportunity;

#[async_trait]
pub trait TriangularArbitrage {
    // async fn check_order_filled(&self, new_order: Order) -> Order;
    async fn send_and_check_filled(&self, new_order: Order, user_state: &UserStateHandle) -> Result<Order, EnumError>;
    async fn forward_trading(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError>;
    async fn reverse_trading(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError>;
    async fn handle_arbitrage(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError>;
}

pub trait MappingSymbol {
    fn symbol_to_enum(&self, symbol: &str) -> Symbol; 
}