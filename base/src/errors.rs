use log::error;
use thiserror::Error;
use serde_json::Value;
use reqwest::Client as HttpClient;

#[derive(Error, Debug)]
pub enum EnumError {
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Serde Json Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to parse JSON reponse: {0}")]
    RequestStatusError(String),
    #[error("Failed to parse JSON reponse: {0}")]
    JsonParsingFailed(String),
    #[error("API key and secret key are required")]
    MissingKeys,
    // Trade Error
    #[error("Can't not find the order with order_id {0} in UserState.user_orders")]
    OrderNotFound(String),
    #[error("Unknown error raised")]
    FirstTriSendError,
    #[error("Unknown error raised")]
    FirstTriFilledError,
    #[error("Unknown error raised")]
    SecondTriSendError,
    #[error("Unknown error raised")]
    SecondTriFilledError,
    #[error("Unknown error raised")]
    ThirdTriSendError,
    #[error("Unknown error raised")]
    ThirdTriFilledError,
    #[error("Unknown error raised")]
    OrderNotFilled,
    #[error("Unknown error raised")]
    UNKNOWN_ERROR,
}

#[derive(Error, Debug)]
pub enum TradeError {
    #[error("Can't not find the order with order_id {0} in UserState.user_orders")]
    OrderNotFound(String),
    #[error("Order Rejected caused by IOC limit price")]
    OrderIOCRejected,
    #[error("Order still not fully filled")]
    OrderNotFilled
}