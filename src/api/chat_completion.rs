use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, DialogMessage},
    api::{EventSender, StreamOutcome, emit, other, post_sse},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::Reasoning,
    tool::Tool,
};

pub(super) fn stream_response(
    agent: &Agent,
    tools: &[Tool],
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut finish_reason = None;
    let mut tool_call_ids = Vec::<Option<String>>::new();

    for data in post_sse(agent, "/chat/completions", &build_request(agent, tools))? {
        let data = data?;
        if data == "[DONE]" {
            use FinishReason::*;
            return match finish_reason {
                Some(Stop) => Ok(StreamOutcome::Finished),
                Some(Length) => Err(AgentMessageError::Length),
                Some(ContentFilter) => Err(AgentMessageError::ContentFilter),
                Some(InsufficientSystemResource) => {
                    Err(AgentMessageError::InsufficientSystemResource)
                }
                Some(ToolCalls) => Ok(StreamOutcome::ToolCalls),
                None => Err(AgentMessageError::Other(
                    "SSE stream finished without a finish reason".to_owned(),
                )),
            };
        }
        let Response { choices } = serde_json::from_str(&data).map_err(other)?;
        let [choice] = <[Choice; 1]>::try_from(choices).map_err(|choices| {
            AgentMessageError::Other(format!(
                "expected one completion choice, got {}",
                choices.len()
            ))
        })?;
        if let Some(reasoning_content) = choice.delta.reasoning_content {
            emit(tx, AgentMessageDelta::ReasoningContent(reasoning_content))?;
        }
        if let Some(content) = choice.delta.content {
            emit(tx, AgentMessageDelta::Content(content))?;
        }
        for tool_call in choice.delta.tool_calls.into_iter().flatten() {
            let index = tool_call.index.unwrap_or_default();
            if tool_call_ids.len() <= index {
                tool_call_ids.resize(index + 1, None);
            }
            if let Some(id) = tool_call.id {
                tool_call_ids[index] = Some(id);
            }
            emit(
                tx,
                AgentMessageDelta::ToolCall {
                    name: tool_call.function.name.unwrap_or_default(),
                    arguments: tool_call.function.arguments,
                    tool_call_id: tool_call_ids[index].clone().unwrap_or_default(),
                },
            )?;
        }
        if let Some(reason) = choice.finish_reason {
            finish_reason = Some(reason);
        }
    }

    Err(AgentMessageError::Other(
        "SSE stream ended before [DONE]".to_owned(),
    ))
}

fn build_request<'a>(agent: &'a Agent, tools: &'a [Tool]) -> Request<'a> {
    let (thinking_type, reasoning_effort) = match agent.reasoning {
        Reasoning::Disabled => ("disabled", None),
        Reasoning::Enabled(effort) => ("enabled", effort.map(|effort| effort.as_str())),
    };

    Request {
        messages: agent.dialog.iter().map(Message::from).collect(),
        model: &agent.model,
        thinking: Thinking {
            r#type: thinking_type,
        },
        reasoning_effort,
        response_format: ResponseFormat { r#type: "text" },
        stream: true,
        tools: tools
            .iter()
            .map(|tool| RequestTool {
                r#type: "function",
                function: RequestFunction {
                    description: &tool.description,
                    name: &tool.name,
                    parameters: &tool.parameters,
                },
            })
            .collect(),
    }
}

#[derive(Serialize)]
struct Request<'a> {
    messages: Vec<Message<'a>>,
    model: &'a str,
    thinking: Thinking,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'static str>,
    response_format: ResponseFormat,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<RequestTool<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
enum Message<'a> {
    System {
        content: &'a str,
    },
    User {
        content: &'a str,
    },
    Assistant {
        content: &'a str,
        #[serde(skip_serializing_if = "str::is_empty")]
        reasoning_content: &'a str,
        // API rejects `"tool_calls": []`; omit the field when there are none.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<RequestToolCall<'a>>,
    },
    Tool {
        content: &'a str,
        tool_call_id: &'a str,
    },
}

impl<'a> From<&'a DialogMessage> for Message<'a> {
    fn from(message: &'a DialogMessage) -> Self {
        match message {
            DialogMessage::System { content } => Self::System { content },
            DialogMessage::User { content } => Self::User { content },
            DialogMessage::Assistant {
                content,
                reasoning_content,
                tool_calls,
            } => Self::Assistant {
                content,
                reasoning_content,
                tool_calls: tool_calls
                    .iter()
                    .map(|tool_call| RequestToolCall {
                        id: &tool_call.id,
                        r#type: "function",
                        function: RequestToolCallFunction {
                            name: &tool_call.name,
                            arguments: &tool_call.arguments,
                        },
                    })
                    .collect(),
            },
            DialogMessage::Tool { id, result } => Self::Tool {
                content: result,
                tool_call_id: id,
            },
        }
    }
}

#[derive(Serialize)]
struct Thinking {
    r#type: &'static str,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: &'static str,
}

#[derive(Serialize)]
struct RequestTool<'a> {
    r#type: &'static str,
    function: RequestFunction<'a>,
}

#[derive(Serialize)]
struct RequestFunction<'a> {
    description: &'a str,
    name: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize)]
struct RequestToolCall<'a> {
    id: &'a str,
    r#type: &'static str,
    function: RequestToolCallFunction<'a>,
}

#[derive(Serialize)]
struct RequestToolCallFunction<'a> {
    name: &'a str,
    arguments: &'a str,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
    finish_reason: Option<FinishReason>,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Deserialize)]
struct ToolCallDelta {
    index: Option<usize>,
    id: Option<String>,
    function: ToolCallFunctionDelta,
}

#[derive(Deserialize)]
struct ToolCallFunctionDelta {
    name: Option<String>,
    #[serde(default)]
    arguments: String,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    InsufficientSystemResource,
}
