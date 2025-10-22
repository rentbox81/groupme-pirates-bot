use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Local, Duration};

#[derive(Clone, Debug)]
pub struct ConversationContext {
    pub user_id: String,
    pub user_name: String,
    pub session_start: DateTime<Local>,
    pub last_activity: DateTime<Local>,
    pub volunteer_intent: bool,
    pub mentioned_bot: bool,
}

pub struct ConversationContextStore {
    contexts: Arc<RwLock<HashMap<String, ConversationContext>>>,
    session_timeout_minutes: i64,
}

impl ConversationContextStore {
    pub fn new(session_timeout_minutes: i64) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            session_timeout_minutes,
        }
    }

    pub async fn create_or_update_context(&self, user_id: String, user_name: String, volunteer_intent: bool, mentioned_bot: bool) {
        let now = Local::now();
        let mut contexts = self.contexts.write().await;
        contexts.insert(user_id.clone(), ConversationContext { user_id, user_name, session_start: now, last_activity: now, volunteer_intent, mentioned_bot });
    }

    pub async fn get_active_context(&self, user_id: &str) -> Option<ConversationContext> {
        self.cleanup_expired_contexts().await;
        let contexts = self.contexts.read().await;
        contexts.get(user_id).cloned()
    }

    pub async fn update_activity(&self, user_id: &str) {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(user_id) { context.last_activity = Local::now(); }
    }

    async fn cleanup_expired_contexts(&self) {
        let now = Local::now();
        let timeout = Duration::minutes(self.session_timeout_minutes);
        let mut contexts = self.contexts.write().await;
        contexts.retain(|_, context| now.signed_duration_since(context.last_activity) < timeout);
    }

    pub async fn clear_context(&self, user_id: &str) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(user_id);
    }
}
