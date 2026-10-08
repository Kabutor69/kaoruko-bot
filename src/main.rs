mod config;
mod db;
mod gemini;
mod handler;

use config::Config;
use db::MemoryStore;
use gemini::GeminiClient;
use handler::Handler;
use serenity::{Client, model::gateway::GatewayIntents};
use tracing::info;

#[tokio::main]
async fn main() {
    // load .env 1st
    dotenvy::dotenv().ok();

    // init structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kaoruko_bot=info,warn".into()),
        )
        .init();

    let cfg = Config::from_env();
    info!(model = %cfg.gemini_model, "Starting Kaoruko bot");

    // MongoDB
    let mongo = mongodb::Client::with_uri_str(&cfg.mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let col = mongo
        .database("kaoruko")
        .collection::<db::Memory>("memory");

    let memory = MemoryStore::new(col);
    memory.ensure_indexes().await;

    // Gemini HTTP client 
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to build HTTP client");

    let gemini = GeminiClient::new(http, cfg.gemini_key, cfg.gemini_model);

    // Discord client 
    let handler = Handler {
        memory,
        gemini,
        chat_channels: cfg.chat_channels,
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&cfg.discord_token, intents)
        .event_handler(handler)
        .await
        .expect("Failed to create Discord client");

    client.start().await.expect("Discord client crashed");
}
