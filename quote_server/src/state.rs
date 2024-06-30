use crate::data_structure::OrderBookL2;
use std::sync::{Arc, RwLock};

#[derive(Debug, Default)]
pub struct SharedState {
    pub order_books: Vec<OrderBookL2>,
}

impl SharedState {
    pub fn update_orderbook(&mut self, new_orderbook: OrderBookL2) {
        if let Some(existing) = self
            .order_books
            .iter_mut()
            .find(|ob| ob.symbol == new_orderbook.symbol)
        {
            *existing = new_orderbook;
        } else {
            self.order_books.push(new_orderbook);
        }
    }
}

pub type SharedStateHandle = Arc<RwLock<SharedState>>;

pub fn create_shared_state() -> SharedStateHandle {
    Arc::new(RwLock::new(SharedState::default()))
}
