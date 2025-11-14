use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_account_profile(
        &self,
        nickname: Option<&str>,
        company: Option<&str>,
        email: Option<&str>,
        college: Option<&str>,
        personal_note: Option<&str>,
    ) -> Result<()> {
        let mut params = serde_json::Map::new();
        if let Some(n) = nickname {
            params.insert("nickname".to_string(), json!(n));
        }
        if let Some(c) = company {
            params.insert("company".to_string(), json!(c));
        }
        if let Some(e) = email {
            params.insert("email".to_string(), json!(e));
        }
        if let Some(c) = college {
            params.insert("college".to_string(), json!(c));
        }
        if let Some(p) = personal_note {
            params.insert("personal_note".to_string(), json!(p));
        }
        self.websocket_send("set_account_profile", json!(params))
            .await?;
        Ok(())
    }
}

