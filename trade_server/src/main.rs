use tokio;
use logger::init_logger;
use rust_decimal_macros::dec;
use base::{models::Order, params::{OrderSide, OrderType, Symbol, SymbolPrecision}};
use trade_server::exchanges::{maicoin::MaiCoin, Exchange};


#[tokio::main]
async fn main() {

    let api_key = Some("APE_KEY".to_string());
    let secret_key = Some("SECRET_KEY".to_string());
    let client = MaiCoin::new(api_key, secret_key);
    // let client = Binance::new(api_key, secret_key);
    
    // let symbol = Symbol::MAX_USDT;
    // match client.get_ticker(&symbol).await {       
    //     Ok(ticker) => println!("{:?}", ticker),
    //     Err(e) => eprintln!("Error fetching {}", e),
    // }

    // match client.get_orderbook(symbol).await {
    //     Ok(ticker) => println!("BTCUSDT: {:?}", ticker),
    //     Err(e) => eprintln!("Error fetching Binance ticker: {}", e),
    // }

    // match client.get_account().await {
    //     Ok(account) => println!("Account Info: {:?}", account),
    //     Err(e) => eprintln!("Error fetching account: {}", e),
    // }

    // match client.get_open_orders(Some("maxusdt")).await {
    //     Ok(account) => println!("Open Orders: {:?}", account),
    //     Err(e) => eprintln!("Error fetching account: {}", e),
    // }
    
    let mut new_order = Order::new_order();
    new_order.symbol = Symbol::BTC_TWD;
    new_order.side = OrderSide::BUY;
    new_order.order_type = OrderType::LIMIT;
    new_order.price = dec!(1950000);
    new_order.amount = dec!(0.006156165646546);

    match client.create_order(new_order.clone()).await {
        Ok(order) => println!("Create Order: {:?}", order),
        Err(e) => eprintln!("Error creating Max order: {}", e),
    }

    // match client.cancel_order("MAX/USDT", "8760404840").await {
    //     Ok(order) => println!("Cancel Order: {:?}", order),
    //     Err(e) => eprintln!("Error canceling Max order: {}", e),
    // }
}