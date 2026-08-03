use bevy::prelude::*;

use crate::agent::{AgentStatus, ApiDialect, DialogMessage, Thinking};

const DEEPSEEK_API_BASE_URL: &str = "https://api.deepseek.com";

#[derive(Component, Default, Clone)]
#[require(AgentStatus)]
pub struct Agent {
    pub api_dialect: ApiDialect,
    pub model: String,
    pub thinking: Thinking,
    pub dialog: Vec<DialogMessage>,
    pub(crate) api_base_url: String,
    pub(crate) api_key: String,
}

impl Agent {
    /// The API key will be read from the environment variable `DEEPSEEK_API_KEY`.
    pub fn new(api_dialect: ApiDialect, model: impl Into<String>, thinking: Thinking) -> Self {
        Agent {
            api_dialect,
            model: model.into(),
            thinking,
            dialog: Vec::new(),
            api_base_url: DEEPSEEK_API_BASE_URL.to_owned(),
            api_key: std::env::var("DEEPSEEK_API_KEY").unwrap(),
        }
    }
}
