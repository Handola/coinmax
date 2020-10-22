use std::env;
use std::fmt::Display;

use reqwest::Error;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{Args, CommandResult, macros::{
    command,
    group,
}, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::{Activity, Ready};
use serenity::prelude::TypeMapKey;
use serenity::static_assertions::_core::fmt::Formatter;
use tokio::time;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const COINBASE_PRO_API: &str = "https://api.pro.coinbase.com";

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

            if let Ok(ticker) = json_result {
                ctx.set_activity(Activity::playing(format!("BTC @ {} €", ticker.price).as_str())).await;
            }
        }
    }
}

struct HttpClient(reqwest::Client);

impl TypeMapKey for HttpClient {
    type Value = HttpClient;
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

#[derive(Deserialize)]
struct Ticker {
    price: String,
}

#[derive(Deserialize)]
struct Stats {
    open: String,
    high: String,
    low: String,
    volume: String,
    last: String,
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Open   = {} €\n\
                   High   = {} €\n\
                   Low    = {} €\n\
                   Volume = {} €\n\
                   Last   = {} €", self.open, self.high, self.low, self.volume, self.last)
    }
}

fn get_product(args: &Args) -> (String, String) {
    let product = match args.current() {
        Some(p) => p,
        None => "BTC",
    }.to_uppercase();

    (product.clone(), format!("{}-EUR", product))
}

async fn get_coinbase<T: DeserializeOwned>(http_client: &HttpClient, endpoint: String) -> Result<T, Error> {
    info!("GET: {}", endpoint);

    http_client.0
        .get(format!("{}{}", COINBASE_PRO_API, endpoint).as_str())
        .send().await
        .unwrap().json::<T>().await
}

#[command]
async fn preis(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let http_client = data.get::<HttpClient>().unwrap();
    let (product, full_product) = get_product(&args);
    let json_result = get_coinbase::<Ticker>(http_client, format!("/products/{}/ticker", full_product)).await;

    let message = match json_result {
        Ok(ticker) => format!("```ini\n1 {} = {} € # Coinbase Pro```", product, ticker.price),
        Err(_) => format!("Sorry, aber {} kenne ich leider nicht! Dafür bin ich wohl zu blöd... Muuuuh!", product),
    };

    msg.channel_id.say(&ctx.http, message).await.unwrap();
    Ok(())
}

#[command]
async fn stats(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let http_client = data.get::<HttpClient>().unwrap();
    let (product, full_product) = get_product(&args);
    let json_result = get_coinbase::<Stats>(http_client, format!("/products/{}/stats", full_product)).await;

    let message = match json_result {
        Ok(stats) => format!("```\
                                    ini\n[ 24h Stats {} ] # Coinbase Pro\n\n\
                                    Open   = {} €\n\
                                    High   = {} €\n\
                                    Low    = {} €\n\
                                    Volume = {} {}\n\
                                    Last   = {} €```", product, stats.open, stats.high, stats.low, stats.volume, product, stats.last),
        Err(_) => format!("Sorry, aber {} kenne ich leider nicht! Dafür bin ich wohl zu blöd... Muuuuh!", product),
    };

    msg.channel_id.say(&ctx.http, message).await.unwrap();
    Ok(())
}
