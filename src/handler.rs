use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tracing::{error, info, warn};

use crate::{db::MemoryStore, gemini::GeminiClient};

// DC hard limit minus a small buffer
const MAX_REPLY_CHARS: usize = 1900;

pub struct Handler {
    pub memory: MemoryStore,
    pub gemini: GeminiClient,
    pub chat_channels: Vec<u64>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(bot = %ready.user.name, model = self.gemini.model(), "Bot online");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let is_dm = msg.guild_id.is_none();
        let mentioned = msg.mentions_me(&ctx).await.unwrap_or(false);
        let in_chat_channel = self.chat_channels.contains(&msg.channel_id.get());

        if !is_dm && !mentioned && !in_chat_channel {
            return;
        }

        // strip bot mention from message
        let bot_id = ctx.cache.current_user().id;
        let text = msg
            .content
            .replace(&format!("<@{bot_id}>"), "")
            .replace(&format!("<@!{bot_id}>"), "")
            .trim()
            .to_string();

        if text.is_empty() {
            let _ = msg.reply(&ctx.http, "Yes? I'm here! 🌸").await;
            return;
        }

        let user_id = msg.author.id.to_string();

        // Commands
        if text.eq_ignore_ascii_case("!forget") {
            self.handle_forget(&ctx, &msg, &user_id).await;
            return;
        }
        if text.eq_ignore_ascii_case("!help") {
            self.handle_help(&ctx, &msg).await;
            return;
        }

        let _typing = msg.channel_id.start_typing(&ctx.http);

        let history = self.memory.load_history(&user_id).await;
        info!(user_id, turns = history.len(), "Asking Gemini");

        match self.gemini.chat(&history, &text).await {
            Ok(mut reply) => {
                if reply.chars().count() > MAX_REPLY_CHARS {
                    warn!(user_id, "Truncating reply to fit Discord limit");
                    reply = reply.chars().take(MAX_REPLY_CHARS).collect();
                }
                if let Err(e) = msg.reply(&ctx.http, &reply).await {
                    error!(error = %e, "Failed to send reply");
                }
                self.memory.remember(&user_id, &text, &reply).await;
            }
            Err(e) => {
                error!(error = %e, user_id, "Gemini error");
                let _ = msg
                    .reply(&ctx.http, "Ah, sorry... my thoughts got tangled. Could you try again? 🌸")
                    .await;
            }
        }
    }
}

impl Handler {
    async fn handle_forget(&self, ctx: &Context, msg: &Message, user_id: &str) {
        let reply = if self.memory.forget(user_id).await {
            "Okay! I've forgotten our past chats. Let's start fresh together 🌸"
        } else {
            "Oh no, I couldn't clear my memory just now... please try again later."
        };
        let _ = msg.reply(&ctx.http, reply).await;
    }

    async fn handle_help(&self, ctx: &Context, msg: &Message) {
        let help = "\
🌸 **Kaoruko Bot — Commands**\n\
`!forget` — Clear our conversation history.\n\
`!help` — Show this message.\n\n\
You can also mention me or send a DM to chat! ✨";
        let _ = msg.reply(&ctx.http, help).await;
    }
}
