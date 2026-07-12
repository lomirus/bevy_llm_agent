use bevy::prelude::*;
use rig::{
    agent::{NoToolConfig, WithBuilderTools},
    client::{CompletionClient, ProviderClient},
    providers::deepseek::{Client, CompletionModel},
};

use super::{Agent, AgentStatus, Thinking};
use crate::tool::{Tool, ToolAdapter};

pub struct AgentBuilder<T = NoToolConfig>(rig::agent::AgentBuilder<CompletionModel, T>);

impl AgentBuilder<NoToolConfig> {
    pub fn new(model: &str) -> AgentBuilder<NoToolConfig> {
        let client = Client::from_env().unwrap();
        let agent_builder = client.agent(model).default_max_turns(usize::MAX - 1);
        AgentBuilder(agent_builder)
    }

    /// Set the thinking effort
    pub fn thinking(self, mode: Thinking) -> Self {
        let thinking_param = match mode {
            Thinking::Off => serde_json::json!({
                "thinking": { "type": "disabled" }
            }),
            Thinking::High => serde_json::json!({
                "thinking": { "type": "enabled" },
                "reasoning_effort": "high"
            }),
            Thinking::Max => serde_json::json!({
                "thinking": { "type": "enabled" },
                "reasoning_effort": "max"
            }),
        };
        AgentBuilder(self.0.additional_params(thinking_param))
    }

    /// Set the system prompt
    pub fn preamble(self, preamble: &str) -> Self {
        AgentBuilder(self.0.preamble(preamble))
    }

    /// Add the tool
    pub fn tool<T: Tool>(self) -> AgentBuilder<WithBuilderTools> {
        AgentBuilder(self.0.tool(ToolAdapter::<T>::new()))
    }

    pub fn build(self) -> Agent {
        Agent {
            agent: self.0.build(),
            dialog: Vec::new(),
            status: AgentStatus::Idle,
        }
    }
}

impl AgentBuilder<WithBuilderTools> {
    pub fn tool<T: Tool>(self) -> AgentBuilder<WithBuilderTools> {
        AgentBuilder(self.0.tool(ToolAdapter::<T>::new()))
    }

    pub fn build(self) -> Agent {
        Agent {
            agent: self.0.build(),
            dialog: Vec::new(),
            status: AgentStatus::Idle,
        }
    }
}
