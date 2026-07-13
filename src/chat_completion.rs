use serde::{Deserialize, Serialize};

use crate::agent::DialogMessage;

#[derive(Serialize)]
pub(crate) struct Request {
    pub(crate) messages: Vec<Message>,
    pub(crate) model: String,
    pub(crate) thinking: Thinking,
    pub(crate) reasoning_effort: Option<String>,
    pub(crate) response_format: ResponseFormat,
    pub(crate) stream: bool,
    pub(crate) tools: Vec<Tool>,
}

#[derive(Serialize)]
pub(crate) struct Message {
    pub(crate) reasoning_content: Option<String>,
    pub(crate) content: String,
    pub(crate) role: String,
    pub(crate) tool_call_id: Option<String>,
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

impl From<DialogMessage> for Message {
    fn from(value: DialogMessage) -> Self {
        match value {
            DialogMessage::System { content } => Self {
                role: "system".to_owned(),
                reasoning_content: None,
                content,
                tool_call_id: None,
                tool_calls: None,
            },
            DialogMessage::User { content } => Self {
                role: "user".to_owned(),
                reasoning_content: None,
                content,
                tool_call_id: None,
                tool_calls: None,
            },
            DialogMessage::Assistant {
                content,
                reasoning_content,
                tool_calls,
            } => Self {
                role: "assistant".to_owned(),
                reasoning_content: Some(reasoning_content),
                content,
                tool_call_id: None,
                tool_calls: Some(
                    tool_calls
                        .into_iter()
                        .map(|tool_call| ToolCall {
                            id: Some(tool_call.id),
                            r#type: Some("function".to_owned()),
                            function: ToolCallFunction {
                                name: Some(tool_call.name),
                                arguments: tool_call.arguments,
                            },
                        })
                        .collect(),
                ),
            },
            DialogMessage::Tool { id, result } => Self {
                role: "tool".to_owned(),
                reasoning_content: None,
                content: result,
                tool_call_id: Some(id),
                tool_calls: None,
            },
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Thinking {
    pub(crate) r#type: String,
}

#[derive(Serialize)]
pub(crate) struct ResponseFormat {
    pub(crate) r#type: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct Tool {
    pub(crate) r#type: String,
    pub(crate) function: ToolFunction,
}

#[derive(Serialize, Clone)]
pub(crate) struct ToolFunction {
    pub(crate) description: String,
    pub(crate) name: String,
    pub(crate) parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ToolCall {
    pub(crate) id: Option<String>,
    pub(crate) r#type: Option<String>,
    pub(crate) function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ToolCallFunction {
    pub(crate) name: Option<String>,
    pub(crate) arguments: String,
}

#[derive(Deserialize)]
pub(crate) struct Response {
    id: String,
    object: String,
    created: u32,
    model: String,
    system_fingerprint: String,
    pub(crate) choices: Vec<Choice>,
    pub(crate) usage: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct Choice {
    index: u8,
    pub(crate) delta: Delta,
    pub(crate) finish_reason: Option<FinishReason>,
}

#[derive(Deserialize)]
pub(crate) struct Delta {
    pub(crate) role: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) reasoning_content: Option<String>,
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    InsufficientSystemResource,
}
