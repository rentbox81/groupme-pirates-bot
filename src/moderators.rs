use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ModeratorsStore {
    moderators: Arc<RwLock<HashSet<String>>>,
}

impl ModeratorsStore {
    pub fn new() -> Self {
        Self {
            moderators: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn add_moderator(&self, user_id: String) {
        let mut mods = self.moderators.write().await;
        mods.insert(user_id);
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
