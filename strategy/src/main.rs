mod triarb_runner;

use triarb_runner::StrategyRunner;
use tokio::sync::mpsc;
use logger::init_logger;
use log::info;
use std::sync::Arc;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    // Initialize logger
    init_logger().expect("Failed to initialize logger");
    info!("Strategy started");
    let (opportunity_sender, mut opportunity_receiver) = mpsc::channel(100);
    // Define the symbols for different strategies
    let symbols_list = vec![
        vec!["btcusdt", "btctwd", "usdttwd"],
        vec!["btcusdt", "btctwd", "usdttwd"],
        vec!["btcusdt", "btctwd", "usdttwd"],
        vec!["btcusdt", "btctwd", "usdttwd"],
        // Add more symbol sets as needed
];

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

    // Await all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    loop{
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
