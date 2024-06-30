use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use crate::data_structure::{OrderBookL2, OrderBookUpdate};
use crate::ws_client::WebSocketClient;
use crate::state::SharedState;

#[derive(Clone)]
pub struct MaiCoinWsClient {
    pub shared_state: Arc<RwLock<SharedState>>
}



impl MaiCoinWsClient {
    pub fn new(shared_state: Arc<RwLock<SharedState>>) -> Self {
        Self { shared_state }
    }

    pub async fn start_orderbook<F>(&self, symbols: Vec<&str>, callback: F)
    where
        F: FnMut(String) + Send + 'static
    {
        let url = "wss://max-stream.maicoin.com/ws";
        let subscribe_message = json!({
            "action": "sub",
            "subscriptions": symbols.iter().map(|symbol| {
                json!({
                    "channel": "book",
                    "market": symbol,
                    "depth": 1
                })
            }).collect::<Vec<_>>(),
            "id": "client1"
        }).to_string();

        let shared_state = self.shared_state.clone();
        let mut callback = callback;

        tokio::spawn(async move {
            let client = WebSocketClient::new(&url, Some(subscribe_message));
            client.start(move |msg| {
                let shared_state = shared_state.clone();
                // let start = Instant::now();
                
                // let mut parse_duration = start.elapsed().as_nanos();
                // let mut acquire_lock = start.elapsed().as_nanos();
                if let Ok(order_book_message) = serde_json::from_str::<MaiCoinOrderBookMessage>(&msg) {
                    // parse_duration = start.elapsed().as_nanos();
                    let mut state = shared_state.write().unwrap();
                    // acquire_lock = start.elapsed().as_nanos();
                    match order_book_message.event.as_str() {
                        "snapshot" => {
                            let start = Instant::now();
                            if let Some(order_book) = state.order_books.iter_mut().find(|ob| ob.symbol == order_book_message.market) {
                                order_book.update_from_snapshot(&order_book_message);
                            } else {
                                let mut new_order_book = OrderBookL2::new(&order_book_message.market, 1000);
                                new_order_book.update_from_snapshot(&order_book_message);
                                state.order_books.push(new_order_book);
                            }
                            let duration = start.elapsed().as_nanos();
                            println!("{} Snapshot update took: {} nanoseconds", order_book_message.market, duration);
                        }
                        "update" => {
                            // let start = Instant::now();
                            if let Some(order_book) = state.order_books.iter_mut().find(|ob| ob.symbol == order_book_message.market) {
                                order_book.update_from_message(order_book_message.clone());
                            }
                            // let duration = start.elapsed().as_nanos();
                            // println!("{} Incremental update took: {} nanoseconds", order_book_message.market, duration);
                        }
                        _ => {
                            println!("Unhandled event: {}", order_book_message.event);
                        }
                    }
                    // Execute the callback function after updating the orderbook
                    
                }
                // let update_book_duration = start.elapsed().as_nanos();
                callback(msg);
                // let total_duration = start.elapsed().as_nanos();
                // println!("Parse:{} acquire:{} update:{} total:{}",  parse_duration, acquire_lock, update_book_duration, total_duration);
            }).await;
        });
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MaiCoinOrderBookMessage {
    #[serde(rename = "c")]
    pub channel: String,
    #[serde(rename = "M")]
    pub market: String,
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "T")]
    pub timestamp: u128,
}

impl OrderBookUpdate for MaiCoinOrderBookMessage {
    fn bids(&self) -> &Vec<[String; 2]> {
        &self.bids
    }

    fn asks(&self) -> &Vec<[String; 2]> {
        &self.asks
    }

    fn timestamp(&self) -> u128 {
        self.timestamp
    }
}