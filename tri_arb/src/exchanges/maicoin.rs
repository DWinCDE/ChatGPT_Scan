use std::time::Instant;
use tokio::sync::RwLock;
use rust_decimal_macros::dec;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::{Arc, RwLockReadGuard};

use base::Config;
use base::models::Order;
use base::utils::{convert_f64_to_decimal, convert_str_to_decimal, symbol_to_enum};
use base::params::{OrderSide, OrderStatus, OrderType, Symbol, TimeInForce};
use base::errors::{EnumError, TradeError};
use trade_server::common::ExchangeInitial;
use trade_server::exchanges::{maicoin::MaiCoin, Exchange};
use user_data::state::{UserState, UserStateHandle, create_user_state};
use user_data::exchanges::maicoin::MaiCoinUserWsClient;
use strategy::ArbitrageOpportunity;
use user_data::ws_client::ExchangeUserClient;
use crate::models::{TriangularArbitrage, MappingSymbol};

#[derive(Clone)]
pub struct MaiCoinTriangularArbitrage {
    pub restful_client: MaiCoin,
    pub user_ws_client: MaiCoinUserWsClient,
    pub user_state: UserStateHandle,
    pub tolerance: f64
}

impl MaiCoinTriangularArbitrage {
    pub fn new(config: Config) -> Self {
        let api_key = Some(config.api_info.api_key.clone());
        let secret_key = Some(config.api_info.secret_key.clone());
        let tolerance = config.settings.protect_tolerance;
        let restful_client = MaiCoin::new(api_key.clone(), secret_key.clone());
        let user_ws_client = MaiCoinUserWsClient::new(api_key.clone(), secret_key.clone());
        let user_state = create_user_state(); 
        Self {
            restful_client,
            user_ws_client,
            user_state,
            tolerance
        }
    }

    pub async fn start(&self) {
        let user_state: Arc<RwLock<UserState>> = self.user_state.clone();
        let user_ws_client: MaiCoinUserWsClient = self.user_ws_client.clone();
        tokio::spawn(async move {
            user_ws_client.start_user_order(user_state).await;
        });
        println!("Start MaiCoin User Orders Websocket Streaming");
    }
}

#[async_trait]
impl TriangularArbitrage for MaiCoinTriangularArbitrage {
    async fn send_and_check_filled(&self, new_order: Order, user_state: &UserStateHandle) -> Result<Order, EnumError> {
        match self.restful_client.create_order(new_order.clone()).await {
            Ok(new_order_response) => {
                println!("{} Send SUCCESS: {:?}", new_order.label.to_string(), new_order);
                for _ in 0..10000 {
                    let read_state: tokio::sync::RwLockReadGuard<UserState> = self.user_state.read().await;
                    // println!("{:?}", read_state);
                    if let Ok(new_order_result) = read_state.query_order(new_order_response.order_id.clone()).await {
                        if let Ok(filled_order) = read_state.check_order_filled(new_order_result).await {
                            return Ok(filled_order.clone());
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                };

                Err(match new_order.label.as_str() {
                    "[#1 Order]" => EnumError::FirstTriFilledError,
                    "[#2 Order]" => EnumError::SecondTriFilledError,
                    "[#3 Order]" => EnumError::ThirdTriFilledError,
                    _ => EnumError::UNKNOWN_ERROR,
                })
            }
            Err(_) => {
                    eprintln!("{} Send FAILED: {:?}", new_order.label.to_string(), new_order);
                    Err(match new_order.label.as_str() {
                    "[#1 Order]" => EnumError::FirstTriSendError,
                    "[#2 Order]" => EnumError::SecondTriSendError,
                    "[#3 Order]" => EnumError::ThirdTriSendError,
                    _ => EnumError::UNKNOWN_ERROR,})
                }            
        }
    }

    async fn forward_trading(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError> {
        let start = Instant::now();
        let max_amount = arbitrage_opportunity.max_amount;
        let first_order_price = arbitrage_opportunity.booktickers[0].ask_price;
        let first_order_amount = max_amount / first_order_price;

        let mut first_order = Order::new_order();
        let duration2: u128 = start.elapsed().as_nanos();
        first_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[0].clone());
        first_order.side = OrderSide::BUY;
        first_order.order_type = OrderType::IOC;
        first_order.price = convert_f64_to_decimal(first_order_price);
        first_order.amount = convert_f64_to_decimal(first_order_amount);
        first_order.label = "[#1 Order]".to_string();

        match self.send_and_check_filled(first_order, &user_state).await {
            Ok(first_order_result) => {
                println!("[#1 Order] Filled SUCCESS: {:?}", first_order_result);
                let second_order_price = arbitrage_opportunity.booktickers[1].bid_price * (1.0 - &self.tolerance);
                let second_order_amount = first_order_result.filled_amount;

                let mut second_order = Order::new_order();
                second_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[1].clone());
                second_order.side = OrderSide::SELL;
                second_order.order_type = OrderType::LIMIT;
                second_order.price = convert_f64_to_decimal(second_order_price);
                second_order.amount = second_order_amount;
                second_order.label = "[#2 Order]".to_string();

                match self.send_and_check_filled(second_order, &user_state).await {
                    Ok(second_order_result) => {
                        println!("[#2 Order] Filled SUCCESS: {:?}", second_order_result);
                        let third_order_price = arbitrage_opportunity.booktickers[2].ask_price * (1.0 + &self.tolerance);
                        let third_order_amount = (second_order_result.filled_amount * second_order_result.filled_price / convert_f64_to_decimal(arbitrage_opportunity.booktickers[2].ask_price));

                        let mut third_order = Order::new_order();
                        third_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[2].clone());
                        third_order.side = OrderSide::BUY;
                        third_order.order_type = OrderType::LIMIT;
                        third_order.price = convert_f64_to_decimal(third_order_price);
                        third_order.amount = third_order_amount;
                        third_order.label = "[#3 Order]".to_string();

                        match self.send_and_check_filled(third_order, &user_state).await {
                            Ok(third_order_result) => {
                                println!("[#3 Order] Filled SUCCESS: {:?}", third_order_result);
                                Ok(json!({
                                "first_trade": first_order_result,
                                "second_trade": second_order_result,
                                "third_trade": third_order_result,
                                }))
                            },
                            Err(err) => {
                                println!("[#3 Order] Filled FAILED");
                                eprintln!("{}", err);
                                Err(err)
                            }
                            
                        }
                    }
                    Err(err) => {
                        println!("[#2 Order] Filled FAILED");
                        eprintln!("{}", err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                println!("[#1 Order] Filled FAILED");
                eprintln!("{}", err);
                Err(err)
            }
        }
    }

    async fn reverse_trading(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError> {
        let start = Instant::now();
        let max_amount = arbitrage_opportunity.max_amount;
        let first_order_price = arbitrage_opportunity.booktickers[1].ask_price;
        let first_order_amount = max_amount * arbitrage_opportunity.booktickers[0].ask_price / first_order_price;
        
        let mut first_order = Order::new_order();
        let duration2: u128 = start.elapsed().as_nanos();
        first_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[1].clone());
        first_order.side = OrderSide::BUY;
        first_order.order_type = OrderType::IOC;
        
        first_order.price = convert_f64_to_decimal(first_order_price);
        first_order.amount = convert_f64_to_decimal(first_order_amount);
        first_order.label = "[#1 Order]".to_string();

        match self.send_and_check_filled(first_order, &user_state).await {
            Ok(first_order_result) => {
                println!("[#1 Order] Filled SUCCESS: {:?}", first_order_result);
                let second_order_price = arbitrage_opportunity.booktickers[2].bid_price * (1.0 - &self.tolerance);
                let second_order_amount = first_order_result.filled_amount;

                let mut second_order = Order::new_order();
                second_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[2].clone());
                second_order.side = OrderSide::SELL;
                second_order.order_type = OrderType::LIMIT;
                second_order.price = convert_f64_to_decimal(second_order_price);
                second_order.amount = second_order_amount;
                second_order.label = "[#2 Order]".to_string();

                match self.send_and_check_filled(second_order, &user_state).await {
                    Ok(second_order_result) => {
                        println!("[#2 Order] Filled SUCCESS: {:?}", second_order_result);
                        let third_order_price = arbitrage_opportunity.booktickers[0].ask_price * (1.0 - &self.tolerance);
                        let third_order_amount = second_order_result.filled_amount * second_order_result.filled_price;

                        let mut third_order = Order::new_order();
                        third_order.symbol = symbol_to_enum(&arbitrage_opportunity.symbols[0].clone());
                        third_order.side = OrderSide::SELL;
                        third_order.order_type = OrderType::LIMIT;
                        third_order.price = convert_f64_to_decimal(third_order_price);
                        third_order.amount = third_order_amount;
                        third_order.label = "[#3 Order]".to_string();

                        match self.send_and_check_filled(third_order, &user_state).await {
                            Ok(third_order_result) => {
                                println!("[#3 Order] Filled SUCCESS: {:?}", third_order_result);
                                Ok(json!({
                                "first_trade": first_order_result,
                                "second_trade": second_order_result,
                                "third_trade": third_order_result,
                                }))
                            },
                            Err(err) => {
                                println!("[#3 Order] Filled FAILED");
                                eprintln!("{}", err);
                                Err(err)
                            }
                            
                        }
                    }
                    Err(err) => {
                        println!("[#2 Order] Filled FAILED");
                        eprintln!("{}", err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                println!("[#1 Order] Filled FAILED");
                eprintln!("{}", err);
                Err(err)
            }
        }
    }

    async fn handle_arbitrage(&self, arbitrage_opportunity: ArbitrageOpportunity, user_state: &UserStateHandle) -> Result<Value, EnumError> {
        let arbitrage_opportunity_clone = arbitrage_opportunity.clone();
        let user_state_clone = user_state.clone();
        let self_clone = self.clone();
        
        match arbitrage_opportunity.direction.as_str() {
            "forward" => {
                tokio::spawn(async move {
                    match self_clone.forward_trading(arbitrage_opportunity_clone, &user_state_clone).await {
                        Ok(result) => {
                            println!("Forward trading completed successfully: {:?}", result);
                        }
                        Err(err) => {
                            eprintln!("Forward trading failed: {:?}", err);
                        }
                    }
                });
            },
            "reverse" => {
                tokio::spawn(async move {
                    match self_clone.reverse_trading(arbitrage_opportunity_clone, &user_state_clone).await {
                        Ok(result) => {
                            println!("Reverse trading completed successfully: {:?}", result);
                        }
                        Err(err) => {
                            eprintln!("Reverse trading failed: {:?}", err);
                        }
                    }
                });
            },
            _ => {
                return Err(EnumError::UNKNOWN_ERROR);
            }
        }

        Ok(json!({"status": "trading_started"}))
    }
}