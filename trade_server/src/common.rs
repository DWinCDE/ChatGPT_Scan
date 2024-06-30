use serde_json::Value;
use async_trait::async_trait;
use std::collections::HashMap;
use reqwest::{Client as HttpClient, Response};
use logger::init_logger;
use base::errors::EnumError;
   
#[async_trait]
pub trait ExchangeSigner {
    fn signature(&self, params: &mut HashMap<String, String>, secret_key: &str);
    fn add_auth_headers(&self, request_builder: reqwest::RequestBuilder, api_key: &str, params: &HashMap<String, String>) -> reqwest::RequestBuilder;
}

#[async_trait]
pub trait ExchangeInitial {
    async fn check_symbol_precision(&self) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Clone)]
pub struct CommonClient<S: ExchangeSigner + Send + Sync> {
    pub client: HttpClient,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub signer: S,
}

impl<S: ExchangeSigner + Send + Sync> CommonClient<S> {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, signer: S) -> Self {
        CommonClient {
            client: HttpClient::new(),
            api_key,
            secret_key,
            signer,
        }
    } 

    pub async fn handle_http_error(&self, resp: Response) -> Result<Value, EnumError> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());

        if status.is_success() {
            match serde_json::from_str::<Value>(&text) {
                Ok(json) => Ok(json),
                Err(_) => Err(EnumError::JsonParsingFailed(text)),
            }
        } else {
            Err(EnumError::RequestStatusError(format!("Status: {}, Body: {}", status, text)))
        }
    }

    pub async fn http_get(&self, url: &str) -> Result<Value, EnumError> {
        match self.client.get(url).send().await {
            Ok(resp) => self.handle_http_error(resp).await,
            Err(err) => Err(EnumError::ReqwestError(err))
        }
    }

    pub async fn sign_http_get(&self, url: &str, params: &mut HashMap<String, String>) -> Result<Value, EnumError> {
        if let (Some(api_key), Some(secret_key)) = (&self.api_key, &self.secret_key) {
            let query = serde_urlencoded::to_string(&params).unwrap();
            self.signer.signature(params, secret_key);
            let signed_url = format!("{}?{}", url, query); 
            let request_builder = self.client.get(signed_url);
            let request_builder = self.signer.add_auth_headers(request_builder, api_key, &params);
            match request_builder.send().await {
                Ok(resp) => self.handle_http_error(resp).await,
                Err(err) => Err(EnumError::ReqwestError(err))
            }
        } else {
            Err(EnumError::MissingKeys)
        }
    }

    pub async fn http_post(&self, url: &str, body: &Value) -> Result<Value, EnumError> {
        match self.client.post(url).json(body).send().await {
            Ok(resp) => self.handle_http_error(resp).await,
            Err(err) => Err(EnumError::ReqwestError(err))
        }
    }

    pub async fn sign_http_post(&self, url: &str, mut params: HashMap<String, String>) -> Result<Value, EnumError> {
        if let (Some(api_key), Some(secret_key)) = (&self.api_key, &self.secret_key) {
            let post_params = params.clone();
            self.signer.signature(&mut params, secret_key);
            let request_builder: reqwest::RequestBuilder = self.client.post(url).json(&post_params);
            let request_builder = self.signer.add_auth_headers(request_builder, api_key, &params);
            match request_builder.send().await {
                Ok(resp) => self.handle_http_error(resp).await,
                Err(err) => Err(EnumError::ReqwestError(err))
            }

        } else {
            Err(EnumError::MissingKeys)
        }
    }
}