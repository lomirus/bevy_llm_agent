use bevy::prelude::*;
use rig::{
    agent::{NoToolConfig, WithBuilderTools},
    client::{CompletionClient, ProviderClient},
    providers::deepseek::{Client, CompletionModel},
};

use crate::tool::{Tool, ToolAdapter};
use super::{Agent, AgentStatus};

pub struct AgentBuilder<T = NoToolConfig>(rig::agent::AgentBuilder<CompletionModel, (), T>);

impl AgentBuilder<NoToolConfig> {
    pub fn new(model: &str) -> AgentBuilder<NoToolConfig> {
        let client = Client::from_env().unwrap();
        let agent_builder = client.agent(model).default_max_turns(usize::MAX - 1);
        AgentBuilder(agent_builder)
    }

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
