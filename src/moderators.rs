use std::collections::HashSet;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize)]
struct ModsJson { mods: Vec<String> }

#[derive(Clone)]
pub struct ModeratorsStore {
    moderators: Arc<RwLock<HashSet<String>>>,
}

impl ModeratorsStore {
    pub fn new() -> Self {
        // Ensure data directory exists
        let _ = std::fs::create_dir_all("data");
        let moderators = if let Ok(content) = std::fs::read_to_string("data/moderators.json") {
            if let Ok(json) = serde_json::from_str::<ModsJson>(&content) {
                json.mods.into_iter().collect()
            } else { HashSet::new() }
        } else { HashSet::new() };
        Self { moderators: Arc::new(RwLock::new(moderators)) }
    }

    pub async fn add_moderator(&self, user_id: String) {
        let mut mods = self.moderators.write().await;
        mods.insert(user_id);
        drop(mods);
        let v = self.list_moderators().await;
        if let Err(e) = std::fs::create_dir_all("data") { tracing::error!("Failed to create data dir: {}", e); }
        let _ = std::fs::write("data/moderators.json", serde_json::to_string(&ModsJson { mods: v }).unwrap_or_default());
    }

    pub async fn remove_moderator(&self, user_id: &str) -> bool {
        let mut mods = self.moderators.write().await;
        let removed = mods.remove(user_id);
        drop(mods);
        if removed {
            let v = self.list_moderators().await;
            if let Err(e) = std::fs::create_dir_all("data") { tracing::error!("Failed to create data dir: {}", e); }
        let _ = std::fs::write("data/moderators.json", serde_json::to_string(&ModsJson { mods: v }).unwrap_or_default());
        }
        removed
    }

    pub async fn is_moderator(&self, user_id: &str) -> bool {
        let mods = self.moderators.read().await;
        mods.contains(user_id)
    }

    pub async fn list_moderators(&self) -> Vec<String> {
        let mods = self.moderators.read().await;
        mods.iter().cloned().collect()
    }

    pub fn is_admin(&self, user_id: &str, admin_user_id: &str) -> bool {
        user_id == admin_user_id
    }

    pub async fn is_authorized(&self, user_id: &str, admin_user_id: &str) -> bool {
        self.is_admin(user_id, admin_user_id) || self.is_moderator(user_id).await
    }
}
