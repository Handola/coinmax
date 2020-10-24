use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult, macros::command};
use serenity::model::channel::Message;
use tracing::error;

use crate::api::{get_product, HttpClient};
use crate::api::coinbase_pro::{get_coinbase, Ticker};

#[command]
pub async fn preis(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let http_client = data.get::<HttpClient>().expect("HttpClient in data");
    let (product, full_product) = get_product(&args);
    let json_result = get_coinbase::<Ticker>(http_client, format!("/products/{}/ticker", full_product)).await;

    let message = match json_result {
        Ok(ticker) => format!("```ini\n1 {} = {} € # Coinbase Pro```", product, ticker.price),
        Err(e) => {
            error!("{}", e);
            format!("Sorry, aber {} kenne ich leider nicht! Dafür bin ich wohl zu blöd... Muuuuh!", product)
        }
    };

    msg.channel_id.say(&ctx.http, message).await?;
    Ok(())
}