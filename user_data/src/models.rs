use serde_json::Value;
use base::models::{Order, CurencyBalance};

pub trait OrderMessageUpdate {
    fn order_update(&self, order_message: &Value) -> Order;
}

pub trait BalanceMessageUpdate {
    fn balance_update(&self, balance_message: &Value) -> CurencyBalance;
}