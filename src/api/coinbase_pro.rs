use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tracing::info;

use crate::api::HttpClient;

const COINBASE_PRO_API: &str = "https://api.pro.coinbase.com";

#[derive(Deserialize)]
pub struct Ticker {
    pub price: String,
}

#[derive(Deserialize)]
pub struct Stats {
    pub open: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub last: String,
}

pub async fn get_coinbase<T: DeserializeOwned>(http_client: &HttpClient, endpoint: String) -> Result<T, Error> {
    info!("GET: {}", endpoint);

    http_client.0
        .get(format!("{}{}", COINBASE_PRO_API, endpoint).as_str())
        .send().await?
        .json::<T>().await
}