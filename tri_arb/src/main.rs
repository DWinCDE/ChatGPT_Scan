use std::env;
use std::sync::Arc;
use async_trait::async_trait;
use base::errors::TradeError;
use logger::init_logger;
use log::*;
use base::utils::load_config;
use quote_server::data_structure::Bookticker;
use strategy::ArbitrageOpportunity;
use trade_server::exchanges::Exchange;
use user_data::ws_client::ExchangeUserClient;
use user_data::state::{create_user_state};
use tokio::sync::mpsc;
use strategy::triarb_runner::StrategyRunner;
use tri_arb::{exchanges::maicoin::MaiCoinTriangularArbitrage, models::TriangularArbitrage};

#[tokio::main]
async fn main() {
    init_logger().expect("Failed to initialize logger");
    info!("Strategy started");

    let config_path = env::current_dir().unwrap().join("config/maicoin.toml");
    println!("{:?}", config_path);
    let config = load_config(config_path.to_string_lossy().to_string());
    let (opportunity_sender, mut opportunity_receiver) = mpsc::channel(100);
    println!("{:?}", config);
    let symbols_list = vec![
        vec!["btcusdt", "btctwd", "usdttwd"],
        vec!["ethusdt", "ethtwd", "usdttwd"],
        // Add more symbol sets as needed
    ];

    // Initialize and start the Triangular Arbitrage client
    let tri_arb_client = MaiCoinTriangularArbitrage::new(config.unwrap());
    tri_arb_client.start().await;

    // Create strategy runners and spawn them as tasks
    let mut handles = vec![];

    for symbols in symbols_list {
        let runner = Arc::new(StrategyRunner::new(symbols, opportunity_sender.clone()));
        let runner_clone = Arc::clone(&runner);
        let handle = tokio::spawn(async move {
            runner_clone.start().await;
        });
        handles.push(handle);
    }

    // Spawn a task to handle received opportunities
    let tri_arb_client_clone = tri_arb_client.clone();
    let opportunity_handle = tokio::spawn(async move {
        while let Some(opportunity) = opportunity_receiver.recv().await {
            tri_arb_client_clone.handle_arbitrage(opportunity, &tri_arb_client_clone.user_state).await;
        }
    });

    // Optionally, you can add another loop to monitor the user state or perform other tasks
    // tokio::spawn(async move {
    //     loop {
    //         let user_state = tri_arb_client.user_state.read().await;
    //         println!("{:?}", user_state);
    //         tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    //     }   
    // });

    // Await all strategy runner tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Await the opportunity handler task
    opportunity_handle.await.unwrap();

    
}
