use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Instant;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookticker {
    pub symbol: String,
    pub bid_price: f64,
    pub bid_quantity: f64,
    pub ask_price: f64,
    pub ask_quantity: f64,
}

pub trait OrderBookUpdate {
    fn bids(&self) -> &Vec<[String; 2]>;
    fn asks(&self) -> &Vec<[String; 2]>;
    fn timestamp(&self) -> u128;
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct OrderLevel {
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug)]
pub struct OrderBookL2 {
    pub symbol: String,
    pub bids: BTreeMap<OrderedFloat<f64>, OrderLevel>, // Sorted in ascending order by key (price), but we'll iterate in reverse
    pub asks: BTreeMap<OrderedFloat<f64>, OrderLevel>, // Sorted in ascending order by key (price)
    pub update_time: u128, // Timestamp of the last update in nanoseconds
    pub max_length: usize, // Maximum number of price levels on one side
}

impl OrderBookL2 {
    pub fn new(symbol: &str, max_length: usize) -> Self {
        Self {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            update_time: Self::current_time(),
            max_length,
        }
    }

    pub fn current_time() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    pub fn update_from_snapshot<U: OrderBookUpdate>(&mut self, snapshot: &U) {
        self.bids.clear();
        self.asks.clear();

        for [price, amount] in snapshot.bids() {
            let price = OrderedFloat(price.parse().unwrap());
            let amount: f64 = amount.parse().unwrap();
            self.bids.insert(
                price,
                OrderLevel {
                    price: price.into_inner(),
                    amount,
                },
            );
        }

        for [price, amount] in snapshot.asks() {
            let price = OrderedFloat(price.parse().unwrap());
            let amount: f64 = amount.parse().unwrap();
            self.asks.insert(
                price,
                OrderLevel {
                    price: price.into_inner(),
                    amount,
                },
            );
        }

        self.update_time = snapshot.timestamp();
        self.truncate_side(true); // Truncate bids
        self.truncate_side(false); // Truncate asks
        self.print_orderbook();
    }

    pub fn update_from_message<U: OrderBookUpdate>(&mut self, update: U) {
        // let start = Instant::now();
        for [price, amount] in update.bids() {
            let price = OrderedFloat(price.parse().unwrap());
            let amount: f64 = amount.parse().unwrap();
            if amount == 0.0 {
                self.bids.remove(&price);
            } else {
                self.bids.insert(
                    price,
                    OrderLevel {
                        price: price.into_inner(),
                        amount,
                    },
                );
            }
        }
        for [price, amount] in update.asks() {
            let price = OrderedFloat(price.parse().unwrap());
            let amount: f64 = amount.parse().unwrap();
            if amount == 0.0 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(
                    price,
                    OrderLevel {
                        price: price.into_inner(),
                        amount,
                    },
                );
            }
        }
        self.update_time = update.timestamp();
        self.truncate_side(true); // Truncate bids
        self.truncate_side(false); // Truncate asks
        // let duration = start.elapsed().as_nanos();
        // println!("update took: {} nanoseconds",duration);
        // let bookticker = self.get_bookticker();
        // println!("{:?}", bookticker)
                                   // self.print_orderbook();
    }
    fn truncate_side(&mut self, is_bids: bool) {
        let side = if is_bids {
            &mut self.bids
        } else {
            &mut self.asks
        };
        while side.len() > self.max_length {
            if is_bids {
                if let Some(first_key) = side.keys().next().cloned() {
                    side.remove(&first_key);
                }
            } else {
                if let Some(last_key) = side.keys().next_back().cloned() {
                    side.remove(&last_key);
                }
            }
        }
    }

    pub fn top_bids(&self, n: usize) -> Vec<OrderLevel> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(&price, level)| OrderLevel {
                price: price.into_inner(),
                amount: level.amount,
            })
            .collect()
    }

    pub fn top_asks(&self, n: usize) -> Vec<OrderLevel> {
        self.asks
            .iter()
            .take(n)
            .map(|(&price, level)| OrderLevel {
                price: price.into_inner(),
                amount: level.amount,
            })
            .collect()
    }
    pub fn get_bookticker(&self) -> Option<Bookticker> {
        let bid = self.bids.iter().next_back();
        let ask = self.asks.iter().next();

        if let (Some(bid), Some(ask)) = (bid, ask) {
            Some(Bookticker {
                symbol: self.symbol.clone(),
                bid_price: bid.1.price,
                bid_quantity: bid.1.amount,
                ask_price: ask.1.price,
                ask_quantity: ask.1.amount,
            })
        } else {
            None
        }
    }
    pub fn print_orderbook(&self) {
        println!("OrderBookL2 for {}: Bid:{:?} Asks{:?}", self.symbol, self.bids, self.asks);
    }
}
