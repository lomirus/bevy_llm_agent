use std::{
    io::{BufRead, BufReader},
    sync::mpsc,
};

use serde::{Deserialize, Serialize};

use crate::{
    agent::{DialogMessage, Thinking as AgentThinking},
    api::{DeltaResult, StreamOutcome, emit_delta},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    tool::Tool as AgentTool,
};

pub(super) fn stream_response(
    dialog: &mut Vec<DialogMessage>,
    model: &str,
    thinking: &AgentThinking,
    tools: &[AgentTool],
    api_base_url: &str,
    api_key: &str,
    tx: &mpsc::Sender<DeltaResult>,
) -> Result<StreamOutcome, AgentMessageError> {
    let body = build_request(dialog, model, *thinking, tools);

    let url = format!("{api_base_url}/chat/completions");
    let mut response = match ureq::post(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .send_json(body)
    {
        Ok(response) => response,
        Err(error) => return Err(AgentMessageError::Other(error.to_string())),
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        let message = if body.is_empty() {
            format!("http status: {status}")
        } else {
            body
        };
        return Err(AgentMessageError::Other(message));
    }

    consume_sse_stream(BufReader::new(response.body_mut().as_reader()), dialog, tx)
}

fn build_request(
    dialog: &[DialogMessage],
    model: &str,
    thinking: AgentThinking,
    tools: &[AgentTool],
) -> Request {
    use AgentThinking::*;

    let reasoning_effort = match model {
        "kimi-k2.5" | "kimi-k2.6" | "kimi-k2.7-code" | "kimi-k2.7-code-highspeed" => None,
        _ => match thinking {
            Off => None,
            Low => Some("low".to_owned()),
            High => Some("high".to_owned()),
            Max => Some("max".to_owned()),
        },
    };
    let thinking = Thinking {
        r#type: match thinking {
            Off => "disabled".to_owned(),
            Low | High | Max => "enabled".to_owned(),
        },
    };

    Request {
        messages: dialog.iter().map(|msg| msg.clone().into()).collect(),
        model: model.to_owned(),
        thinking,
        reasoning_effort,
        response_format: ResponseFormat {
            r#type: "text".to_owned(),
        },
        stream: true,
        tools: tools
            .iter()
            .map(|tool| Tool {
                r#type: "function".to_owned(),
                function: ToolFunction {
                    description: tool.description.clone(),
                    name: tool.name.clone(),
                    parameters: tool.parameters.clone(),
                },
            })
            .collect(),
    }
}

fn consume_sse_stream(
    reader: impl BufRead,
    dialog: &mut Vec<DialogMessage>,
    tx: &mpsc::Sender<DeltaResult>,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut finish_reason = None;
    let mut tool_call_ids = Vec::<Option<String>>::new();

    for line in reader.lines() {
        let line = line.map_err(|error| AgentMessageError::Other(error.to_string()))?;
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
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
        let data: Response = serde_json::from_str(data)
            .map_err(|error| AgentMessageError::Other(error.to_string()))?;
        if data.choices.len() != 1 {
            return Err(AgentMessageError::Other(format!(
                "expected one completion choice, got {}",
                data.choices.len()
            )));
        }
        let choice = &data.choices[0];
        if let Some(reasoning_content) = &choice.delta.reasoning_content {
            emit_delta(
                dialog,
                tx,
                AgentMessageDelta::ReasoningContent(reasoning_content.to_owned()),
            );
        }
        if let Some(content) = &choice.delta.content {
            emit_delta(dialog, tx, AgentMessageDelta::Content(content.to_owned()));
        }
        if let Some(tool_calls) = &choice.delta.tool_calls {
            for tool_call in tool_calls {
                let index = tool_call.index.unwrap_or_default();
                if tool_call_ids.len() <= index {
                    tool_call_ids.resize(index + 1, None);
                }
                if let Some(id) = &tool_call.id {
                    tool_call_ids[index] = Some(id.clone());
                }
                emit_delta(
                    dialog,
                    tx,
                    AgentMessageDelta::ToolCall {
                        name: tool_call.function.name.clone().unwrap_or_default(),
                        arguments: tool_call.function.arguments.clone(),
                        tool_call_id: tool_call
                            .id
                            .clone()
                            .or_else(|| tool_call_ids[index].clone())
                            .unwrap_or_default(),
                    },
                );
            }
        }
        if let Some(reason) = choice.finish_reason {
            finish_reason = Some(reason);
        }
    }

    Err(AgentMessageError::Other(
        "SSE stream ended before [DONE]".to_owned(),
    ))
}

#[derive(Serialize)]
struct Request {
    messages: Vec<Message>,
    model: String,
    thinking: Thinking,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    response_format: ResponseFormat,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool>,
}

#[derive(Serialize)]
struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
    content: String,
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
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
                reasoning_content: if reasoning_content.is_empty() {
                    None
                } else {
                    Some(reasoning_content)
                },
                content,
                tool_call_id: None,
                // API rejects `"tool_calls": []`; omit the field when there are none.
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(
                        tool_calls
                            .into_iter()
                            .map(|tool_call| ToolCall {
                                index: None,
                                id: Some(tool_call.id),
                                r#type: Some("function".to_owned()),
                                function: ToolCallFunction {
                                    name: Some(tool_call.name),
                                    arguments: tool_call.arguments,
                                },
                            })
                            .collect(),
                    )
                },
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
struct Thinking {
    r#type: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize, Clone)]
struct Tool {
    r#type: String,
    function: ToolFunction,
}

#[derive(Serialize, Clone)]
struct ToolFunction {
    description: String,
    name: String,
    parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
struct ToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    id: Option<String>,
    r#type: Option<String>,
    function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone)]
struct ToolCallFunction {
    name: Option<String>,
    arguments: String,
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
    tool_calls: Option<Vec<ToolCall>>,
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
