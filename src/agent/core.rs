use bevy::prelude::*;

use crate::{
    agent::{AgentStatus, ApiDialect, DialogMessage, Thinking},
    provider::ProviderBuilder,
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
    /// The API key is read from the environment variable specified by the provider.
    pub fn new(provider_builder: ProviderBuilder<'_>, thinking: Thinking) -> Result<Self, String> {
        let provider = provider_builder.build(thinking)?;
        Ok(Agent {
            api_dialect: provider.api_dialect,
            model: provider.model.to_owned(),
            thinking: provider.thinking,
            dialog: Vec::new(),
            api_base_url: provider.base_url.to_owned(),
            api_key: provider.api_key,
        })
    }
}
