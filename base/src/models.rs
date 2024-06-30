use serde::Serialize;
use crate::params::{Symbol, OrderSide, OrderType, OrderStatus, TimeInForce};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, Serialize)] // Ensure OrderStatus supports PartialEq
pub struct Order {
    pub symbol: Symbol,
    pub order_id: String,
    pub client_id: String,
    pub label: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Decimal,
    pub amount: Decimal,
    pub status: OrderStatus,
    pub filled_price: Decimal,
    pub filled_amount: Decimal,
    pub remaining_amount: Decimal,
    pub created_ts: u64,
    pub updated_ts: u64,
}

#[derive(Debug, Clone)]
pub struct CurencyBalance {
    pub currency: String,
    pub available: String,
    pub locked: String,
    pub staked: String,
    pub updated_ts: u64,
}


impl Order {
    pub fn new_order() -> Self {
        Order {
            symbol: Symbol::BTC_USDT,
            order_id: "order_id".to_string(),
            client_id: "client_id".to_string(),
            label: "label".to_string(),
            side: OrderSide::UNKNOWN_ORDER_SIDE,
            order_type: OrderType::UNKNOWN_ORDER_TYPE,
            time_in_force: TimeInForce::GTC,
            price: dec!(0.00),
            amount: dec!(0.00),
            status: OrderStatus::UNKNOWN_STATUS,
            filled_price: dec!(0.00),
            filled_amount: dec!(0.00),
            remaining_amount: dec!(0.00),
            created_ts: 0,
            updated_ts: 0,
        }
    }
}