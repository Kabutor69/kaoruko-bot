use futures::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime},
    options::IndexOptions,
    Collection, IndexModel,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::config::HISTORY_LIMIT;

// one message turn
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
    pub user_id: String,
    pub role: String, // "user" or "model"
    pub text: String,
    pub ts: DateTime,
}

// handles all MongoDB reads/writes for memory
#[derive(Clone)]
pub struct MemoryStore {
    col: Collection<Memory>,
}

impl MemoryStore {
    pub fn new(col: Collection<Memory>) -> Self {
        Self { col }
    }

    // Create the (user_id, ts) index
    pub async fn ensure_indexes(&self) {
        let idx = IndexModel::builder()
            .keys(doc! { "user_id": 1, "ts": -1 })
            .options(IndexOptions::builder().build())
            .build();

        match self.col.create_index(idx).await {
            Ok(r) => info!(index = %r.index_name, "MongoDB index ready"),
            Err(e) => error!(error = %e, "MongoDB index creation failed"),
        }
    }

    // fetch last N turns oldest-first, trim leading model turns for Gemini
    pub async fn load_history(&self, user_id: &str) -> Vec<Memory> {
        let cursor = self
            .col
            .find(doc! { "user_id": user_id })
            .sort(doc! { "ts": -1 })
            .limit(HISTORY_LIMIT)
            .await;

        let mut items: Vec<Memory> = match cursor {
            Ok(c) => c.try_collect().await.unwrap_or_default(),
            Err(e) => {
                error!(error = %e, "MongoDB find failed");
                vec![]
            }
        };

        items.reverse(); // flip to chronological
        while items.first().map(|m| m.role != "user").unwrap_or(false) {
            items.remove(0);
        }
        items
    }

    // save both sides of a turn (+1 ms offset keeps sort stable)
    pub async fn remember(&self, user_id: &str, user_text: &str, bot_text: &str) {
        let now = DateTime::now();
        let later = DateTime::from_millis(now.timestamp_millis() + 1);

        let rows = vec![
            Memory { user_id: user_id.into(), role: "user".into(), text: user_text.into(), ts: now },
            Memory { user_id: user_id.into(), role: "model".into(), text: bot_text.into(), ts: later },
        ];

        if let Err(e) = self.col.insert_many(rows).await {
            error!(error = %e, "MongoDB insert failed");
        }
    }

    // wipe all turns for a user, returns true on success
    pub async fn forget(&self, user_id: &str) -> bool {
        match self.col.delete_many(doc! { "user_id": user_id }).await {
            Ok(res) => {
                info!(user_id, deleted = res.deleted_count, "Cleared user memory");
                true
            }
            Err(e) => {
                error!(error = %e, "MongoDB delete failed");
                false
            }
        }
    }
}
