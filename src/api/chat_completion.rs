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
) -> StreamOutcome {
    use AgentThinking::*;

    let body = Request {
        messages: dialog.iter().map(|msg| msg.clone().into()).collect(),
        model: model.to_owned(),
        thinking: Thinking {
            r#type: match thinking {
                Off => "disabled".to_owned(),
                High | Max => "enabled".to_owned(),
            },
        },
        reasoning_effort: match thinking {
            Off => None,
            High => Some("high".to_owned()),
            Max => Some("max".to_owned()),
        },
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
    };

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
        Err(error) => return StreamOutcome::Error(AgentMessageError::Other(error.to_string())),
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        let message = if body.is_empty() {
            format!("http status: {status}")
        } else {
            body
        };
        return StreamOutcome::Error(AgentMessageError::Other(message));
    }

    consume_sse_stream(BufReader::new(response.body_mut().as_reader()), dialog, tx)
}

fn consume_sse_stream(
    reader: impl BufRead,
    dialog: &mut Vec<DialogMessage>,
    tx: &mpsc::Sender<DeltaResult>,
) -> StreamOutcome {
    for line in reader.lines().map_while(Result::ok) {
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        if data == "[DONE]" {
            unreachable!()
        }
        let data: Response = serde_json::from_str(data).unwrap();
        assert_eq!(data.choices.len(), 1);
        let choice = &data.choices[0];
        if let Some(reason) = &choice.finish_reason {
            use FinishReason::*;
            return match reason {
                Stop => StreamOutcome::Finished,
                Length => StreamOutcome::Error(AgentMessageError::Length),
                ContentFilter => StreamOutcome::Error(AgentMessageError::ContentFilter),
                InsufficientSystemResource => {
                    StreamOutcome::Error(AgentMessageError::InsufficientSystemResource)
                }
                ToolCalls => StreamOutcome::ToolCalls,
            };
        }
        if let Some(reasoning_content) = &choice.delta.reasoning_content {
            assert!(choice.delta.content.is_none());
            assert!(choice.delta.tool_calls.is_none());
            emit_delta(
                dialog,
                tx,
                AgentMessageDelta::ReasoningContent(reasoning_content.to_owned()),
            );
        } else if let Some(content) = &choice.delta.content {
            assert!(choice.delta.tool_calls.is_none());
            emit_delta(dialog, tx, AgentMessageDelta::Content(content.to_owned()));
        } else if let Some(tool_calls) = &choice.delta.tool_calls {
            assert_eq!(tool_calls.len(), 1);
            let tool_call = &tool_calls[0];
            match tool_call.id {
                Some(_) => assert!(tool_call.function.name.is_some()),
                None => assert!(tool_call.function.name.is_none()),
            }
            emit_delta(
                dialog,
                tx,
                AgentMessageDelta::ToolCall {
                    name: tool_call.function.name.clone().unwrap_or_default(),
                    arguments: tool_call.function.arguments.clone(),
                    tool_call_id: tool_call.id.clone().unwrap_or_default(),
                },
            );
        }
    }
    unreachable!()
}

#[derive(Serialize)]
struct Request {
    messages: Vec<Message>,
    model: String,
    thinking: Thinking,
    reasoning_effort: Option<String>,
    response_format: ResponseFormat,
    stream: bool,
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
    #[expect(unused)]
    id: String,
    #[expect(unused)]
    object: String,
    #[expect(unused)]
    created: u32,
    #[expect(unused)]
    model: String,
    #[expect(unused)]
    system_fingerprint: String,
    choices: Vec<Choice>,
    #[expect(unused)]
    usage: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    #[expect(unused)]
    index: u8,
    delta: Delta,
    finish_reason: Option<FinishReason>,
}

#[derive(Deserialize)]
struct Delta {
    #[expect(unused)]
    role: Option<String>,
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    InsufficientSystemResource,
}
