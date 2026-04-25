// Action native : envoi de message Telegram

use crate::registry::ActionRegistry;

pub struct SendTelegramAction {
    pub bot_token: String,
}

impl ActionRegistry for SendTelegramAction {
    fn name(&self) -> &str {
        "send_telegram"
    }
}
