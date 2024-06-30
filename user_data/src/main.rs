use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::runtime::Runtime;
use tokio::time::{self, Duration};
use base::params::{Symbol, OrderType, OrderSide};

use user_data::exchanges::maicoin::MaiCoinUserWsClient;
use user_data::state::{create_user_state};
use user_data::ws_client::ExchangeUserClient;
use trade_server::exchanges::{maicoin::MaiCoin, Exchange};


#[tokio::main]
async fn main() {
    let rt = Runtime::new().unwrap();
    let api_key = "API_KEY".to_string();
    let secret_key = "SECRET_KEY".to_string();

    let user_state = create_user_state();
    let user_client = MaiCoinUserWsClient::new(Some(api_key.clone()), Some(secret_key.clone()));
    let restful_client = MaiCoin::new(Some(api_key.clone()), Some(secret_key.clone()));

    let user_order_state = user_state.clone();
    let user_balance_state = user_state.clone();
    tokio::spawn(async move {
        user_client.start_user_balance(user_balance_state).await;
    });
    println!("Initialization complete. The service is now running.");
   
    // let symbol = Symbol::MAX_USDT;
    // let orderSide = OrderSide::SELL;
    // let orderType = OrderType::LIMIT;
    // let order =  restful_client.create_order(&symbol, &orderSide, &orderType, 100.0, Some(0.5)).await.unwrap();
    // println!("{:?}", order.order_id);

    loop {
            let read_state = user_state.read().await;
            // Example pseudo-code for accessing state
            println!("Current user state: {:?}", read_state);
            // println!("{:?}", read_state.query_order(order.order_id.to_string()));
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    };
}