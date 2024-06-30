use super::Exchange;
use crate::common::{CommonClient, ExchangeSigner, EnumError};
use crate::models::{ExchangeResponseMapper, Ticker, Orderbook, OrderbookEntry};

use base::utils::match_market;
use base::params::{ExchangeParams, Symbol, OrderSide, OrderType};
use serde_json::Value;
use async_trait::async_trait;
use std::collections::HashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex::encode;
use chrono::Utc;

type HmacSha256 = Hmac<Sha256>;

pub struct Binance {
    pub client: CommonClient<BinanceSigner>,
    pub base_url: String,
}

impl ExchangeParams for Binance {
    fn market(&self, symbol: &Symbol) -> String {
        symbol.to_string().replace("_", "").to_uppercase()
    }

    fn orderType(&self, order_type: &OrderType) -> String {
        match order_type {
            OrderType::LIMIT => "LIMIT".to_string(),
            OrderType::MARKET => "MARKET".to_string(),
            OrderType::IOC => "IOC".to_string(),
            OrderType::POST_ONLY => "POST_ONLY".to_string(),
        }
    }

    fn orderSide(&self, side: &OrderSide) -> String {
        match side {
            OrderSide::BUY => "BUY".to_string(),
            OrderSide::SELL => "SELL".to_string(),
        }
    }

    fn orderId(&self, order_id: &str) -> String {
        order_id.to_string()
    }
}

impl ExchangeResponseMapper for Binance {
    fn safe_ticker(&self, response: &Value) -> Ticker {
        Ticker {
            bid: response["bidPrice"].as_str().unwrap().parse().unwrap(),
            ask: response["askPrice"].as_str().unwrap().parse().unwrap(),
            timestamp: Utc::now().timestamp_millis() as u64,
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
            timestamp: Utc::now().timestamp_millis() as u64,
            update_id: response["last_update_id"].as_u64().unwrap(),
        }
    }
}

pub struct BinanceSigner;

#[async_trait]
impl ExchangeSigner for BinanceSigner {
    fn signature(&self, params: &mut HashMap<String, String>, secret_key: &str) {
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        let query = serde_urlencoded::to_string(&sorted_params).unwrap();
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).unwrap();
        mac.update(query.as_bytes());
        let signature = encode(mac.finalize().into_bytes());
        params.insert("signature".to_string(), signature);
    }

    fn add_auth_headers(&self, request_builder: reqwest::RequestBuilder, api_key: &str, params: &HashMap<String, String>) -> reqwest::RequestBuilder {
        request_builder.header("X-MBX-APIKEY", api_key)
    }
}

impl Binance {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Binance {
            client: CommonClient::new(api_key, secret_key, BinanceSigner),
            base_url: "https://api.binance.com".to_string(),
        }
    }
}

#[async_trait]
impl Exchange for Binance {
    async fn get_ticker(&self, symbol: &Symbol) -> Result<Ticker, EnumError> {
        let market = self.market(symbol);
        let url = format!("{}/api/v3/ticker/price?symbol={}", self.base_url, market);
        match self.client.http_get(&url).await {
            Ok(response) => Ok(self.safe_ticker(&response)),
            Err(err) => Err(err)
        }
        
    }

    async fn get_orderbook(&self, symbol: &Symbol) -> Result<Orderbook, EnumError> {
        let market = self.market(symbol);
        let url = format!("{}/api/v3/depth?symbol={}", self.base_url, market);
        match self.client.http_get(&url).await {
            Ok(response) => Ok(self.safe_orderbook(&response)),
            Err(err) => Err(err)
        }
    }

    async fn get_account(&self) -> Result<Value, EnumError> {
        let url = format!("{}/api/v3/account", self.base_url);
        let mut params = HashMap::new();
        params.insert("timestamp".to_string(), Utc::now().timestamp_millis().to_string());
        self.client.sign_http_get(&url, &mut params).await
    }

    async fn get_open_orders(&self, symbol: &Symbol) -> Result<Value, EnumError> {
        let url = format!("{}/api/v3/openOrders", self.base_url);
        let market = self.market(symbol);
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), market);
        params.insert("timestamp".to_string(), Utc::now().timestamp_millis().to_string());
        self.client.sign_http_get(&url, &mut params).await
    }

    async fn create_order(&self, symbol: &Symbol, side: &OrderSide, order_type: &OrderType, quantity: &f64, price: &Option<f64>) -> Result<Value, EnumError> {
        let url = format!("{}/api/v3/order", self.base_url);
        let market = self.market(symbol);
        let orderSide = self.orderSide(side);
        let orderType = self.orderType(order_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), market);
        params.insert("side".to_string(), orderSide);
        params.insert("type".to_string(), orderType);
        params.insert("quantity".to_string(), quantity.to_string());
        if let Some(price) = price {
            params.insert("price".to_string(), price.to_string());
        }
        params.insert("timestamp".to_string(), Utc::now().timestamp_millis().to_string());
        self.client.sign_http_post(&url, params).await
    }

    async fn cancel_order(&self, symbol: &Symbol, order_id: &str) -> Result<Value, EnumError> {
        let url = format!("{}/api/v3/order", self.base_url);
        let market = self.market(symbol);
        let orderId = self.orderId(order_id);
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), market);
        params.insert("orderId".to_string(), orderId);
        params.insert("timestamp".to_string(), Utc::now().timestamp_millis().to_string());
        self.client.sign_http_post(&url, params).await
    }
}