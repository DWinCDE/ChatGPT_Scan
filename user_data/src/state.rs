use std::sync::Arc;
use base::params::OrderStatus;
use tokio::sync::RwLock;
use std::collections::HashMap;
use base::errors::EnumError;
use base::models::{Order, CurencyBalance};

#[derive(Debug, Default)]
pub struct UserState {
    pub account_orders: HashMap<String, Order>,
    pub account_balances: HashMap<String, CurencyBalance>,
}

impl UserState {
    pub async fn query_order(&self, order_id: String) -> Result<&Order, EnumError> {
        match self.account_orders.get(&order_id) {
            Some(order) => Ok(order),
            None => Err(EnumError::OrderNotFound(order_id)),
        }
    }

    pub async fn check_order_filled<'a>(&self, order: &'a Order) -> Result<&'a Order, EnumError> {
        match order.status {
            OrderStatus::FILLED => Ok(order),
            _ => Err(EnumError::OrderNotFilled),
        }
    }
}

pub type UserStateHandle = Arc<RwLock<UserState>>;

pub fn create_user_state() -> UserStateHandle {
    Arc::new(RwLock::new(UserState::default()))
}