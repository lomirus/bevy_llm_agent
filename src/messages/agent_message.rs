use bevy::prelude::*;

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: Result<AgentMessageDelta, AgentMessageError>,
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
    Finish,
}

#[derive(Clone, Debug)]
pub enum AgentMessageError {
    Length,
    ContentFilter,
    InsufficientSystemResource,
    Other(String),
}
