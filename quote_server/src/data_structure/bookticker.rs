use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub price: f64,
    pub amount: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderBookLevel {
    pub price: f64,
    pub orders: Vec<Order>,
}

#[derive(Debug)]
pub struct Orderbook {
    pub symbol: String,
    pub bids: BTreeMap<f64, Vec<Order>>, // Sorted in ascending order by key (price), but we'll iterate in reverse
    pub asks: BTreeMap<f64, Vec<Order>>, // Sorted in ascending order by key (price)
    pub update_time: u64, // Timestamp of the last update
    pub max_length: usize, // Maximum number of price levels on one side
    pub total_bid_volume: f64,
    pub total_ask_volume: f64,
    pub order_count: usize, // Total number of orders
}

impl Orderbook {
    pub fn new(symbol: &str, max_length: usize) -> Self {
        Self {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            update_time: Self::current_time(),
            max_length,
            total_bid_volume: 0.0,
            total_ask_volume: 0.0,
            order_count: 0,
        }
    }

    pub fn current_time() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    }

    pub fn add_order(&mut self, order: Order, is_bid: bool) {
        let book_side = if is_bid { &mut self.bids } else { &mut self.asks };
        book_side.entry(order.price).or_insert_with(Vec::new).push(order.clone());
        
        if is_bid {
            self.total_bid_volume += order.amount;
        } else {
            self.total_ask_volume += order.amount;
        }
        
        self.order_count += 1;
        self.update_time = Self::current_time();
        
        if book_side.len() > self.max_length {
            let key_to_remove = if is_bid {
                *book_side.iter().next().unwrap().0
            } else {
                *book_side.iter().next_back().unwrap().0
            };
            let removed_orders = book_side.remove(&key_to_remove).unwrap();
            for removed_order in removed_orders {
                if is_bid {
                    self.total_bid_volume -= removed_order.amount;
                } else {
                    self.total_ask_volume -= removed_order.amount;
                }
                self.order_count -= 1;
            }
        }
    }

    pub fn remove_order(&mut self, order_id: &str, price: f64, is_bid: bool) {
        let book_side = if is_bid { &mut self.bids } else { &mut self.asks };
        if let Some(orders) = book_side.get_mut(&price) {
            let mut removed_volume = 0.0;
            orders.retain(|order| {
                if order.order_id == order_id {
                    removed_volume = order.amount;
                    false
                } else {
                    true
                }
            });
            if orders.is_empty() {
                book_side.remove(&price);
            }

            if is_bid {
                self.total_bid_volume -= removed_volume;
            } else {
                self.total_ask_volume -= removed_volume;
            }
            self.order_count -= 1;
            self.update_time = Self::current_time();
        }
    }

    pub fn update(&mut self, bids: Vec<Order>, asks: Vec<Order>) {
        for bid in bids {
            self.add_order(bid, true);
        }
        for ask in asks {
            self.add_order(ask, false);
        }
    }

    pub fn top_bids(&self, n: usize) -> Vec<OrderBookLevel> {
        self.bids.iter().rev().take(n).map(|(&price, orders)| OrderBookLevel { price, orders: orders.clone() }).collect()
    }

    pub fn top_asks(&self, n: usize) -> Vec<OrderBookLevel> {
        self.asks.iter().take(n).map(|(&price, orders)| OrderBookLevel { price, orders: orders.clone() }).collect()
    }
}
