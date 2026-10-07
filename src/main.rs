use std::env;

use serenity::{
    async_trait,
    model::channel::Message,
    prelude::*,
    Client,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content == "hello" {
            if let Err(error) = msg.channel_id.say(&ctx.http, "Hello!! 🌸").await {
                println!("Error sending message: {error}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN not found");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    client
        .start()
        .await
        .expect("Error starting client");
}
