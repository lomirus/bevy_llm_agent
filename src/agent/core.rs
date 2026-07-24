use bevy::prelude::*;

use crate::agent::{AgentStatus, DialogMessage, Thinking};

#[derive(Component, Default, Clone)]
#[require(AgentStatus)]
pub struct Agent {
    pub model: String,
    pub thinking: Thinking,
    pub dialog: Vec<DialogMessage>,
    pub(crate) api_key: String,
}

impl Agent {
    /// The API key will be read from the environment variable `DEEPSEEK_API_KEY`.
    pub fn new(model: impl Into<String>, thinking: Thinking) -> Self {
        Agent {
            model: model.into(),
            thinking,
            dialog: Vec::new(),
            api_key: std::env::var("DEEPSEEK_API_KEY").unwrap(),
        }
    }
}
