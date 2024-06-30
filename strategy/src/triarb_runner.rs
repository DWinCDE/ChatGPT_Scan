// use strategy::{calculate_arbitrage, fetch_data, ArbitrageOpportunity};
// use crate::{ArbitrageOpportunity, fetch_data};
use std::sync::{Arc, RwLock};
use quote_server::state::{create_shared_state, SharedStateHandle};
use quote_server::maicoin::MaiCoinWsClient;
use quote_server::ws_client::ExchangeClient;
use tokio::sync::mpsc;
use quote_server::data_structure::{OrderBookL2, Bookticker};
use log::{info, error};
use std::time::Instant;

const TRADING_FEE: f64 = 0.00105;

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub description: String,
    pub value: f64,
    pub symbols: Vec<String>,
    pub booktickers: Vec<Bookticker>,
    pub direction: String,
    pub max_amount: f64,
}

pub fn fetch_data(symbols: Vec<&str>, shared_state: &SharedStateHandle) -> Result<Vec<Bookticker>, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let state = shared_state.read().unwrap();
    let mut quotes = Vec::new();

    for symbol in symbols.iter() {
        if let Some(order_book) = state.order_books.iter().find(|ob| ob.symbol == *symbol) {
            if let Some(quote) = order_book.get_bookticker() {
                quotes.push(quote);
            } else {
                error!("No valid bid/ask price for symbol {}", symbol);
                return Err("No valid bid/ask price".into());
            }
        } else {
            error!("No order book found for symbol {}", symbol);
            return Err("No order book found".into());
        }
    }
    let duration = start.elapsed().as_nanos();
    // println!("Fetch data {} cost nanos ", duration);
    Ok(quotes)
}

pub fn calculate_arbitrage(quotes: Vec<Bookticker>) -> Option<ArbitrageOpportunity> {
    if quotes.len() != 3 {
        log::error!("Expected 3 quotes, got {}", quotes.len());
        return None;
    }

    // Example logic for triangular arbitrage
    // Let's assume the quotes are in the order of A/B, B/C, and C/A
    let ab = quotes[0].ask_price * (1.0 + TRADING_FEE);
    let bc = quotes[1].bid_price * (1.0 - TRADING_FEE);
    let ca = quotes[2].ask_price * (1.0 + TRADING_FEE);

    let forward_opportunity = (1.0 / ab) * bc * (1.0 / ca);

    // Reverse direction: A/C, C/B, B/A
    let ac = quotes[2].bid_price * (1.0 - TRADING_FEE);
    let cb = quotes[1].ask_price * (1.0 + TRADING_FEE);
    let ba = quotes[0].bid_price * (1.0 - TRADING_FEE);

    let reverse_opportunity = ac * (1.0 / cb) * ba;

    let max_depth_ab = quotes[0].ask_quantity * quotes[0].ask_price;
    let max_depth_bc = quotes[1].bid_quantity * (quotes[1].bid_price / quotes[2].ask_price);
    let max_depth_ca = quotes[2].ask_quantity;

    let max_amount_forward = max_depth_ab.min(max_depth_bc).min(max_depth_ca);

    let max_depth_ac = quotes[2].bid_quantity;
    let max_depth_cb = quotes[1].ask_quantity * (quotes[1].ask_price / quotes[2].bid_price);
    let max_depth_ba = quotes[0].bid_quantity * quotes[0].bid_price;

    let max_amount_reverse = max_depth_ac.min(max_depth_cb).min(max_depth_ba);
    println!("Arbs Value forward:{} reverse:{}", {forward_opportunity}, {reverse_opportunity});
    if forward_opportunity > reverse_opportunity && forward_opportunity > 1.0 {
        Some(ArbitrageOpportunity {
            description: format!("Forward arbitrage opportunity: {} -> {} -> {}", quotes[0].symbol, quotes[1].symbol, quotes[2].symbol),
            value: forward_opportunity,
            symbols: vec![quotes[0].symbol.clone(), quotes[1].symbol.clone(), quotes[2].symbol.clone()],
            booktickers: quotes,
            direction: "forward".to_string(),
            max_amount: max_amount_forward,
        })
    } else if reverse_opportunity > 1.0 {
        Some(ArbitrageOpportunity {
            description: format!("Reverse arbitrage opportunity: {} -> {} -> {}", quotes[2].symbol, quotes[1].symbol, quotes[0].symbol),
            value: reverse_opportunity,
            symbols: vec![quotes[2].symbol.clone(), quotes[1].symbol.clone(), quotes[0].symbol.clone()],
            booktickers: quotes,
            direction: "reverse".to_string(),
            max_amount: max_amount_reverse,
        })
    } else {
        None
    }
}
pub struct StrategyRunner {
    symbols: Vec<&'static str>,
    shared_state: SharedStateHandle,
    maicoin_client: MaiCoinWsClient,
    opportunity_sender: mpsc::Sender<ArbitrageOpportunity>,
}

impl StrategyRunner {
    pub fn new(symbols: Vec<&'static str>, opportunity_sender: mpsc::Sender<ArbitrageOpportunity>) -> Self {
        let shared_state = create_shared_state();
        let maicoin_client = MaiCoinWsClient::new(shared_state.clone());
        Self {
            symbols,
            shared_state,
            maicoin_client,
            opportunity_sender,
        }
    }

    pub async fn start(&self) {
        let symbols = self.symbols.clone();
        let shared_state = self.shared_state.clone();
        let maicoin_client = self.maicoin_client.clone();
        let opportunity_sender = self.opportunity_sender.clone();

        // Start the WebSocket client and listen to order book updates
        let symbols_to_move = symbols.clone();
        maicoin_client.start_orderbook(symbols_to_move, move |msg| {
            // Process the message and update the order book
            // println!("Received callback message: {}", msg);  // Debugging print statement
            // Fetch the updated order book data
            if let Ok(quotes) = fetch_data(symbols.clone(), &shared_state) {
                // Calculate arbitrage opportunities
                if let Some(arbitrage_opportunity) = calculate_arbitrage(quotes) {
                    println!("{:?}", arbitrage_opportunity);
                    // Send the arbitrage opportunity to the trade server
                    if let Err(e) = opportunity_sender.try_send(arbitrage_opportunity) {
                        log::error!("Failed to send arbitrage opportunity: {}", e);
                    }
                }
            }
        }).await;
    }
}
