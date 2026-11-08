use bevy::prelude::*;

use crate::{
    agent::{AgentBuilder, AgentStatus, ApiDialect, DialogMessage, Thinking},
    provider::ModelPreset,
};

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
    /// Configure an agent using a model's defaults.
    ///
    /// ```no_run
    /// use bevy_llm_agent::{agent::{Agent, Thinking}, provider::deepseek};
    ///
    /// let agent = Agent::builder(deepseek::V4_FLASH)
    ///     .thinking(Thinking::Off)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder(preset: ModelPreset<'_>) -> AgentBuilder<'_> {
        AgentBuilder::new(preset)
    }
}
