use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult, macros::command};
use serenity::model::channel::Message;

use crate::api::{get_product, HttpClient};
use crate::api::coinbase_pro::{get_coinbase, Stats};

#[command]
pub async fn stats(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let http_client = data.get::<HttpClient>().expect("HttpClient in data");
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

    msg.channel_id.say(&ctx.http, message).await?;
    Ok(())
}
