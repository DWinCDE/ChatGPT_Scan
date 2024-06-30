mod logger;

use logger::init_logger;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    init_logger().expect("Failed to initialize logger");

    log::info!("Application started");

    // Logging from different libraries

    sleep(Duration::from_secs(1)).await;

    log::info!("Application finished");
}