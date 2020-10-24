use std::env;

use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::gateway::{Activity, Ready};
use tokio::time;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    price::*,
    stats::*,
};

use crate::api::coinbase_pro::{get_coinbase, Ticker};
use crate::api::HttpClient;

mod commands;
mod api;

#[group]
#[commands(preis, stats)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        let mut interval = time::interval(time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            let data = ctx.data.read().await;
            let http_client = data.get::<HttpClient>().unwrap();
            let json_result = get_coinbase::<Ticker>(http_client, "/products/BTC-EUR/ticker".to_string()).await;

            match json_result {
                Ok(ticker) => ctx.set_activity(Activity::playing(format!("BTC @ {} €", ticker.price).as_str())).await,
                Err(e) => error!("{}", e),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("max "))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::new(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to create client");

    let http_client = HttpClient(reqwest::Client::builder().user_agent("Coinmax").build().unwrap());

    {
        let mut data = client.data.write().await;
        data.insert::<HttpClient>(http_client);
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}