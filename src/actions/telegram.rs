// Action native : envoi de message Telegram via l'API HTTP

use crate::registry::{ActionContext, ActionHandler, HookSignal};
use reqwest::blocking::Client;
use serde::Serialize;

pub struct SendTelegramAction {
    pub bot_token: String,
    pub default_chat_id: String,
}

impl SendTelegramAction {
    pub fn new(bot_token: String, default_chat_id: String) -> Self {
        Self { bot_token, default_chat_id }
    }
}

impl ActionHandler for SendTelegramAction {
    fn name(&self) -> &str { "send_telegram" }

    fn handle(&self, ctx: &ActionContext) -> Result<HookSignal, String> {
        let chat_id = ctx.payload
            .get("chat_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.default_chat_id);

        let text = ctx.payload
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("(message vide)");

        #[derive(Serialize)]
        struct TgPayload<'a> {
            chat_id: &'a str,
            text: String,
            parse_mode: &'a str,
        }

        let body = TgPayload {
            chat_id,
            text: format!(
                "🔄 Workflow **{}**\nObjet `{}` : `{}` → `{}`\n\n{}",
                ctx.workflow_id, ctx.object_id, ctx.current_state, ctx.target_state, text
            ),
            parse_mode: "Markdown",
        };

        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;

        let resp = client.post(&url).json(&body).send()
            .map_err(|e| format!("Telegram send error: {}", e))?;

        if resp.status().is_success() {
            log::info!("[telegram] Sent → chat {}", chat_id);
            Ok(HookSignal::Continue)
        } else {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            log::warn!("[telegram] HTTP {}: {}", status, body);
            Err(format!("Telegram HTTP {}: {}", status, body))
        }
    }
}
