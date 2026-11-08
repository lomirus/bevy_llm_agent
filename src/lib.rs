pub mod agent;
mod api;
pub mod messages;
mod plugin;
pub mod provider;
pub mod tool;

pub use crate::plugin::LlmAgentPlugin;
pub use bevy_llm_agent_macros::agent_tool;

/// The commonly used types: `use bevy_llm_agent::prelude::*;`.
pub mod prelude {
    pub use crate::{
        LlmAgentPlugin,
        agent::{Agent, DialogMessage, Responding},
        agent_tool,
        messages::{
            agent_message::{AgentMessage, AgentMessageDelta, AgentMessageError},
            user_message::UserMessage,
        },
        tool::{AgentTools, ToolInput},
    };
}
