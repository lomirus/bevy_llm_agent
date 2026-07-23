pub mod agent;
mod chat_completion;
mod plugin;
pub mod tool;

pub use crate::plugin::LlmAgentPlugin;
use crate::tool::{ToolInvocation, ToolTrait};
use bevy::prelude::*;

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: AgentMessageDelta,
}

pub enum AgentMessageDelta {
    Content(String),
    ReasoningContent(String),
    ToolCall {
        name: String,
        arguments: String,
        tool_call_id: String,
    },
    ToolResult {
        content: String,
        tool_call_id: String,
    },
    Finish(FinishReason),
}

pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    InsufficientSystemResource,
}

#[derive(Message)]
pub struct UserMessage {
    pub entity: Entity,
    pub prompt: String,
}

impl UserMessage {
    pub fn new(entity: Entity, prompt: impl Into<String>) -> Self {
        UserMessage {
            entity,
            prompt: prompt.into(),
        }
    }
}

pub const DEEPSEEK_V4_FLASH: &str = "deepseek-v4-flash";
pub const DEEPSEEK_V4_PRO: &str = "deepseek-v4-pro";

pub trait AppExt {
    fn add_agent_tool<T: ToolTrait>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn add_agent_tool<T: ToolTrait>(&mut self) -> &mut Self {
        self.add_message::<ToolInvocation<T>>()
            .add_systems(FixedUpdate, T::boxed_system())
    }
}
