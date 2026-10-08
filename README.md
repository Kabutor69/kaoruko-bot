# Kaoruko Bot

A Discord chatbot that acts as **Kaoruko Waguri** from *The Fragrant Flower Blooms with Dignity*.

[Invite to your server](https://discord.com/oauth2/authorize?client_id=1557322613959823360)

---

## Stack

| Layer | Tech |
|---|---|
| Language | Rust (2021 edition) |
| Discord | [Serenity](https://github.com/serenity-rs/serenity) v0.12 |
| AI | Google Gemini API (`generateContent`) |
| Memory | MongoDB Atlas |
---

## Setup

### 1. Clone & install Rust
```bash
git clone https://github.com/kabutor69/kaoruko-bot
cd kaoruko-bot
```

### 2. Create `.env`
```env
DISCORD_TOKEN=your_discord_bot_token
GEMINI_API_KEY=your_gemini_api_key
MONGODB_URI=mongodb+srv://user:pass@cluster.mongodb.net/
GEMINI_MODEL=gemini-3.5-flash-lite   
CHAT_CHANNEL_IDS=       # optional
```

### 3. Run locally
```bash
cargo run
```

Set `RUST_LOG=debug` for verbose logs.

---

## Bot behaviour

- Replies in **DMs** always
- Replies when **@mentioned & reply** in a server
- Replies to **every message** in channels listed in `CHAT_CHANNEL_IDS`
- Stays fully in character as Kaoruko , loves sweets 
- full character system prompt is in [`src/config.rs`](src/config.rs)

## Commands

| Command | What it does |
|---|---|
| `!forget` | Clears your conversation history |
| `!help` | Shows the command list |

---

## Project structure

```
src/
├── main.rs      — startup, wires everything together
├── config.rs    — env vars, constants, system prompt
├── db.rs        — MongoDB memory (read / write / forget)
├── gemini.rs    — Gemini API client
└── handler.rs   — Discord event handler & commands
```

---
