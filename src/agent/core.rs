use bevy::prelude::*;

use crate::{
    agent::{AgentBuilder, AgentStatus, ApiDialect, DialogMessage},
    provider::{ModelPreset, Reasoning, Thinking},
};

#[derive(Component, Default, Clone)]
#[require(AgentStatus)]
pub struct Agent {
    pub api_dialect: ApiDialect,
    pub model: String,
    pub dialog: Vec<DialogMessage>,
    pub(crate) reasoning: Reasoning,
    pub(crate) api_base_url: String,
    pub(crate) api_key: String,
}

impl Agent {
    /// Configure an agent using a model's defaults.
    ///
    /// The preset determines which thinking options [`AgentBuilder::thinking`] accepts.
    ///
    /// ```no_run
    /// use bevy_llm_agent::{agent::Agent, provider::deepseek};
    ///
    /// let agent = Agent::builder(deepseek::V4_FLASH)
    ///     .thinking(deepseek::FlashThinking::Off)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder<T: Thinking>(preset: ModelPreset<'_, T>) -> AgentBuilder<'_, T> {
        AgentBuilder::new(preset)
    }
}
