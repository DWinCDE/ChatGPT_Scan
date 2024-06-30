use logger::init_logger;
use quote_server::data_structure::OrderBookL2;
use quote_server::maicoin::MaiCoinWsClient;
use quote_server::state::{create_shared_state, SharedStateHandle};
use quote_server::ws_client::ExchangeClient;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() {
    // Create a runtime for the async tasks
    init_logger().expect("Failed to initialize logger");
    log::info!("Quote server started");

    // Create the shared state
    let shared_state = create_shared_state();

    // Create the MaiCoin WebSocket client
    let maicoin_client = MaiCoinWsClient::new(shared_state);

    // Define the symbols to subscribe to
    let symbols = vec!["btcusdt"];

    maicoin_client
        .start_orderbook(symbols.clone(), move |msg| {
            // println!("Received callback message: {}", msg);
            // Process the message with your strategy logic here
        })
        .await;
    loop {
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
