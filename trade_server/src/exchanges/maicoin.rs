use super::Exchange;
use crate::common::{CommonClient, ExchangeSigner, ExchangeInitial};
use crate::models::{ExchangeResponseMapper, Ticker, Orderbook, OrderbookEntry};
use base::utils::{symbol_to_enum, convert_str_to_decimal};
use base::params::{ExchangeParams, OrderSide, OrderStatus, OrderType, Symbol, SymbolPrecision, TimeInForce};
use base::models::{Order};
use base::errors::EnumError;
use serde_json::{json, Value};
use async_trait::async_trait;
use std::collections::HashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex::encode;
use chrono::Utc;
use base64;
use rust_decimal::Decimal;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct MaiCoin {
    pub client: CommonClient<MaiCoinSigner>,
    pub base_url: String,
    pub symbol_precision: HashMap<Symbol, SymbolPrecision>
}

impl ExchangeParams for MaiCoin {
    fn market(&self, symbol: Symbol) -> String {
        symbol.to_string().replace("_", "").to_lowercase()
    }

    fn orderType(&self, order_type: OrderType) -> String {
        match order_type {
            OrderType::LIMIT => "limit".to_string(),
            OrderType::MARKET => "market".to_string(),
            OrderType::IOC => "ioc_limit".to_string(),
            OrderType::POST_ONLY => "post_only".to_string(),
            OrderType::UNKNOWN_ORDER_TYPE => "unknown_order_type".to_string(),
        }
    }

    fn orderSide(&self, side: OrderSide) -> String {
        match side {
            OrderSide::BUY => "buy".to_string(),
            OrderSide::SELL => "sell".to_string(),
            OrderSide::UNKNOWN_ORDER_SIDE => "unknown_order_side".to_string(),
        }
    }

    fn orderId(&self, order_id: &str) -> String {
        order_id.to_string()
    }
}

impl ExchangeResponseMapper for MaiCoin {
    fn safe_ticker(&self, response: &Value) -> Ticker {
        Ticker {
            bid: response.get("buy").unwrap().to_string(),
            ask: response.get("sell").unwrap().to_string(),
            timestamp: response.get("at").unwrap().as_u64().unwrap(),
        }
    }

    fn safe_orderbook(&self, response: &Value) -> Orderbook {
        let bids = response["bids"].as_array().unwrap().iter().map(|b| OrderbookEntry {
            price: b[0].as_str().unwrap().parse().unwrap(),
            quantity: b[1].as_str().unwrap().parse().unwrap(),
        }).collect();

        let asks = response["asks"].as_array().unwrap().iter().map(|a| OrderbookEntry {
            price: a[0].as_str().unwrap().parse().unwrap(),
            quantity: a[1].as_str().unwrap().parse().unwrap(),
        }).collect();

        Orderbook {
            bids,
            asks,
            timestamp: response["timestamp"].as_u64().unwrap() * 1000,
            update_id: response["last_update_id"].as_u64().unwrap(),
        }
    }

    fn safe_order(&self, response: &Value) -> Order {
        let order_side = match response.get("side") {
            Some(side_value) => {
                match side_value.as_str() {
                    Some("buy") => OrderSide::BUY,
                    Some("sell") => OrderSide::SELL,
                    _ => OrderSide::UNKNOWN_ORDER_SIDE
                }
            }
            None => OrderSide::UNKNOWN_ORDER_SIDE
        };

        let order_type = match response.get("ord_type") {
            Some(order_type_value) => {
                match order_type_value.as_str() {
                    Some("market") => OrderType::MARKET,
                    Some("stop_market") => OrderType::MARKET,
                    Some("post_only") => OrderType::LIMIT,
                    Some("limit") => OrderType::LIMIT,
                    Some("ioc_limit") => OrderType::LIMIT,
                    Some("stop_limit") => OrderType::LIMIT,
                    _ => OrderType::UNKNOWN_ORDER_TYPE
                }
            }
            None => OrderType::UNKNOWN_ORDER_TYPE
        };

        let time_in_force = match response.get("ord_type") {
            Some(time_in_force_value) => {
                match time_in_force_value.as_str() {
                    Some("market") => TimeInForce::GTC,
                    Some("stop_market") => TimeInForce::GTC,
                    Some("post_only") => TimeInForce::MAKER_ONLY,
                    Some("limit") => TimeInForce::GTC,
                    Some("ioc_limit") => TimeInForce::IOC,
                    Some("stop_limit") => TimeInForce::GTC,
                    _ => TimeInForce::UNKNOWN_TIMEINFORCE
                }
            }
            None => TimeInForce::UNKNOWN_TIMEINFORCE
        };

        let order_status: OrderStatus = match response.get("state") {
            Some(order_status_value) => {
                match order_status_value.as_str() {
                    Some("wait") => OrderStatus::NEW,
                    Some("cancel") => OrderStatus::CANCEL,
                    Some("done") => OrderStatus::FILLED,
                    _ => OrderStatus::UNKNOWN_STATUS
                }
            }
            None => OrderStatus::UNKNOWN_STATUS
        };
        Order {
            symbol: symbol_to_enum(&response.get("market").unwrap().as_str().unwrap().to_string()),
            order_id: response.get("id").unwrap().to_string(),
            client_id: response.get("client_oid").unwrap().to_string(),
            label: "-".to_string(),
            side: order_side,
            order_type: order_type,
            time_in_force: time_in_force,            
            price: convert_str_to_decimal(response.get("price").unwrap().as_str().unwrap()),
            amount: convert_str_to_decimal(response.get("volume").unwrap().as_str().unwrap()),
            status: order_status,
            filled_price: convert_str_to_decimal(response.get("avg_price").unwrap().as_str().unwrap()),
            filled_amount: convert_str_to_decimal(response.get("executed_volume").unwrap().as_str().unwrap()),
            remaining_amount: convert_str_to_decimal(response.get("remaining_volume").unwrap().as_str().unwrap()),
            created_ts: response.get("created_at_in_ms").unwrap().as_u64().unwrap(),
            updated_ts: response.get("updated_at_in_ms").unwrap().as_u64().unwrap(),
        }
        
    }
}

#[derive(Clone)]
pub struct MaiCoinSigner;

#[async_trait]
impl ExchangeSigner for MaiCoinSigner {
    fn signature(&self, params: &mut HashMap<String, String>, secret_key: &str) {
        let json_params = serde_json::to_string(&params).unwrap();
        let payload = base64::encode(json_params.as_bytes());
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).unwrap();        
        mac.update(payload.as_bytes());
        let signature = encode(mac.finalize().into_bytes());
        params.insert("payload".to_string(), payload);
        params.insert("signature".to_string(), signature);
    }

    fn add_auth_headers(&self, request_builder: reqwest::RequestBuilder, api_key: &str, params: &HashMap<String, String>) -> reqwest::RequestBuilder {
        if let (Some(payload), Some(signature)) = (params.get("payload"), params.get("signature")) {
            request_builder
                .header("X-MAX-ACCESSKEY", api_key)
                .header("X-MAX-PAYLOAD", payload)
                .header("X-MAX-SIGNATURE", signature)
        } else {
            // Handle the case where 'payload' or 'signature' are missing
            // This is important to avoid panics due to missing keys
            panic!("'payload' or 'signature' missing in params"); 
        }
        
    }
}

impl MaiCoin {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            client: CommonClient::new(api_key, secret_key, MaiCoinSigner),
            base_url: "https://max-api.maicoin.com".to_string(),
            symbol_precision: HashMap::new()
        }
    }
}

#[async_trait]
impl Exchange for MaiCoin {
    async fn get_exchange_info(&self) -> Result<Value, EnumError> {
        let url = format!("{}/api/v2/markets", self.base_url);
        match self.client.http_get(&url).await {
            Ok(response) => Ok(response),
            Err(err) => Err(err)
        }
    }

    async fn get_ticker(&self, symbol: Symbol) -> Result<Ticker, EnumError> {
        let market = self.market(symbol);
        let url = format!("{}/api/v2/tickers/{}", self.base_url, market);
        match self.client.http_get(&url).await {
            Ok(response) => Ok(self.safe_ticker(&response)),
            Err(err) => Err(err)
        }
    }

    async fn get_orderbook(&self, symbol: Symbol) -> Result<Orderbook, EnumError> {
        let market = self.market(symbol.clone());
        let url = format!("{}/api/v2/depth?market={}", self.base_url, market);
        match self.client.http_get(&url).await {
            Ok(response) => Ok(self.safe_orderbook(&response)),
            Err(err) => Err(err)
        }
    }

    async fn get_account(&self) -> Result<Value, EnumError> {
        let ts = Utc::now().timestamp_millis();
        let path = "/api/v2/members/accounts";
        let mut params = HashMap::new();
        params.insert("nonce".to_string(), ts.to_string());
        params.insert("path".to_string(), path.to_string());
        let url = format!("{}{}", self.base_url, path);
        self.client.sign_http_get(&url, &mut params).await
    }

    async fn get_open_orders(&self, symbol: Symbol) -> Result<Value, EnumError> {
        let ts: i64 = Utc::now().timestamp_millis();
        let path = "/api/v2/orders";
        let market = self.market(symbol.clone());
        let mut params = HashMap::new();
        params.insert("nonce".to_string(), format!("{}", ts));
        params.insert("path".to_string(), path.to_string());
        params.insert("market".to_string(), market);
        let url = format!("{}{}", self.base_url, path);
        self.client.sign_http_get(&url, &mut params).await
    }

    async fn create_order(&self, new_order: Order) -> Result<Order, EnumError> {
        let ts: i64 = Utc::now().timestamp_millis();
        let path = "/api/v2/orders";
        // let market = self.market(symbol);
        let market = self.market(new_order.symbol.clone());
        let orderSide = self.orderSide(new_order.side.clone());
        let orderType = self.orderType(new_order.order_type.clone());

        let mut params = HashMap::new();
        params.insert("nonce".to_string(), ts.to_string());
        params.insert("path".to_string(), path.to_string());        
        params.insert("market".to_string(), market);
        params.insert("side".to_string(), orderSide);
        params.insert("ord_type".to_string(), orderType);
        params.insert("volume".to_string(), new_order.amount.clone().to_string());
        params.insert("price".to_string(), new_order.price.clone().to_string());
        let url = format!("{}{}", self.base_url, path);
        match self.client.sign_http_post(&url, params).await {
            Ok(response) => Ok(self.safe_order(&response)),
            Err(err) => {
                eprintln!("placing order error: {:?}", err);
                Err(err)
            }
        }
    }

    async fn cancel_order(&self, symbol: Symbol, order_id: &str) -> Result<Value, EnumError> {
        let ts: i64 = Utc::now().timestamp_millis();
        let path = "/api/v2/order/delete";
        let market = self.market(symbol.clone());
        let orderId = self.orderId(order_id);

        let mut params = HashMap::new();
        params.insert("nonce".to_string(), ts.to_string());
        params.insert("path".to_string(), path.to_string());
        params.insert("market".to_string(), market);
        params.insert("id".to_string(), orderId);
        let url = format!("{}{}", self.base_url, path);
        self.client.sign_http_post(&url, params).await
    }
}

// #[async_trait]
// impl ExchangeInitial for MaiCoin {
//     async fn check_symbol_precision(&self) -> Result<(), EnumError::UNKNOWN_ERROR> {
//         let resp = self.get_exchange_info().await?;
//         println!("{:?}", resp);
//         Ok(())
//     }
// }