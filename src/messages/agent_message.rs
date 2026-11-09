use std::{error::Error, fmt};

use bevy::prelude::*;

/// Progress of an agent's response, in the order it happens.
#[derive(Message, Clone, Debug)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: AgentMessageDelta,
}

#[derive(Clone, Debug)]
pub enum AgentMessageDelta {
    /// The next fragment of the reply.
    Content(String),
    /// The next fragment of the reasoning, for models that expose it.
    ReasoningContent(String),
    /// The model started calling a tool.
    ToolCallStart { id: String, name: String },
    /// The next fragment of the JSON arguments of tool call `id`.
    ToolCallArguments { id: String, arguments: String },
    /// Tool call `id` ran and returned `content` to the model.
    ToolResult { id: String, content: String },
    /// The response ended, successfully or not. Nothing else is sent until the next prompt.
    Finished(Result<(), AgentMessageError>),
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AgentMessageError {
    /// The reply reached the output token limit.
    Length,
    /// The reply was stopped by the provider's content filter.
    ContentFilter,
    /// The provider ran out of resources while generating the reply.
    InsufficientSystemResource,
    Other(String),
}

impl fmt::Display for AgentMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length => f.write_str("the reply reached the output token limit"),
            Self::ContentFilter => f.write_str("the reply was stopped by the content filter"),
            Self::InsufficientSystemResource => {
                f.write_str("the provider ran out of resources for the reply")
            }
            Self::Other(message) => f.write_str(message),
        }
    }
}

impl Error for AgentMessageError {}
