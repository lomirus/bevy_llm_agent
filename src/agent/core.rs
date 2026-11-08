use bevy::prelude::*;

use crate::{
    agent::{AgentBuilder, ApiDialect, DialogMessage},
    provider::{ModelPreset, Reasoning, ReasoningFormat, Thinking},
};

#[derive(Component, Default, Clone)]
pub struct Agent {
    pub api_dialect: ApiDialect,
    pub model: String,
    pub dialog: Vec<DialogMessage>,
    pub(crate) reasoning: Reasoning,
    pub(crate) reasoning_format: ReasoningFormat,
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
    /// let agent = Agent::builder(deepseek::FLASH)
    ///     .thinking(deepseek::Thinking::Off)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder<T: Thinking>(preset: ModelPreset<'_, T>) -> AgentBuilder<'_, T> {
        AgentBuilder::new(preset)
    }
}
