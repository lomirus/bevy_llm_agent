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
        /// Non-empty when a tool call starts; empty for argument-only deltas.
        name: String,
        /// The next fragment of the tool arguments.
        arguments: String,
        /// Identifies the tool call when supplied by the streaming event.
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
