use serenity::framework::standard::Args;
use serenity::prelude::*;

pub mod coinbase_pro;

pub struct HttpClient(pub reqwest::Client);

impl TypeMapKey for HttpClient {
    type Value = HttpClient;
}

pub fn get_product(args: &Args) -> (String, String) {
    let product = match args.current() {
        Some(p) => p,
        None => "BTC",
    }.to_uppercase();

    (product.clone(), format!("{}-EUR", product))
}