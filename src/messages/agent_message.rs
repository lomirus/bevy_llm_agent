use bevy::prelude::*;

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: AgentMessageDelta,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    InsufficientSystemResource,
}
