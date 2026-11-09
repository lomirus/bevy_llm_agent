//! The Anthropic Messages API.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    agent::{Agent, ApiDialect, DialogMessage},
    api::{EventSender, StreamEvent, StreamOutcome, emit, other, post_sse_with_headers, send},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::Reasoning,
    tool::Tool as AgentTool,
};

const API_VERSION: &str = "2026-10-21";
/// The output limit, thinking included. Streaming keeps long replies clear of HTTP timeouts.
const MAX_TOKENS: u32 = 64_000;

pub(super) fn stream_response(
    agent: &Agent,
    tools: &[AgentTool],
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let headers = [
        ("x-api-key", agent.api_key.as_str()),
        ("anthropic-version", API_VERSION),
    ];
    let events = post_sse_with_headers(agent, "/messages", &headers, &build_request(agent, tools))?;
    read_events(events, tx)
}

/// A content block being streamed, with the JSON of a tool call's input so far.
struct OpenBlock {
    block: Value,
    input_json: String,
}

fn read_events(
    events: impl Iterator<Item = Result<String, AgentMessageError>>,
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut open_blocks = HashMap::<usize, OpenBlock>::new();
    let mut stop_reason = None;

    for data in events {
        let event: Event = serde_json::from_str(&data?).map_err(other)?;
        match event {
            Event::ContentBlockStart {
                index,
                content_block,
            } => {
                if !content_block.is_object() {
                    return Err(other(format!("content block {index} is not an object")));
                }
                if content_block["type"] == "tool_use" {
                    let (Some(id), Some(name)) =
                        (content_block["id"].as_str(), content_block["name"].as_str())
                    else {
                        return Err(other("a tool call started without an id or a name"));
                    };
                    emit(
                        tx,
                        AgentMessageDelta::ToolCallStart {
                            id: id.to_owned(),
                            name: name.to_owned(),
                        },
                    )?;
                }
                open_blocks.insert(
                    index,
                    OpenBlock {
                        block: content_block,
                        input_json: String::new(),
                    },
                );
            }
            Event::ContentBlockDelta { index, delta } => {
                let Some(open) = open_blocks.get_mut(&index) else {
                    return Err(other(format!("received a delta for unknown block {index}")));
                };
                match delta {
                    Delta::TextDelta { text } => {
                        append(&mut open.block, "text", &text);
                        emit(tx, AgentMessageDelta::Content(text))?;
                    }
                    Delta::ThinkingDelta { thinking } => {
                        append(&mut open.block, "thinking", &thinking);
                        emit(tx, AgentMessageDelta::ReasoningContent(thinking))?;
                    }
                    Delta::SignatureDelta { signature } => {
                        append(&mut open.block, "signature", &signature);
                    }
                    Delta::InputJsonDelta { partial_json } => {
                        open.input_json.push_str(&partial_json);
                        if !partial_json.is_empty() {
                            let id = open.block["id"].as_str().unwrap_or_default().to_owned();
                            emit(
                                tx,
                                AgentMessageDelta::ToolCallArguments {
                                    id,
                                    arguments: partial_json,
                                },
                            )?;
                        }
                    }
                    Delta::Other => {}
                }
            }
            Event::ContentBlockStop { index } => {
                let Some(OpenBlock {
                    mut block,
                    input_json,
                }) = open_blocks.remove(&index)
                else {
                    return Err(other(format!("received the end of unknown block {index}")));
                };
                if !input_json.trim().is_empty() {
                    // Input that does not parse is still run, so the tool reports it to the
                    // model; the call is sent back with empty input, which the API accepts.
                    block["input"] = serde_json::from_str(&input_json)
                        .unwrap_or_else(|_| Value::Object(Default::default()));
                }
                // Thinking blocks are only valid in place, so the whole turn is sent back.
                send(tx, StreamEvent::RawOutput(ApiDialect::Messages, block))?;
            }
            Event::MessageDelta { delta } => {
                if delta.stop_reason.is_some() {
                    stop_reason = delta.stop_reason;
                }
            }
            Event::MessageStop => {
                return match stop_reason.as_deref() {
                    Some("end_turn" | "stop_sequence") => Ok(StreamOutcome::Finished),
                    Some("tool_use") => Ok(StreamOutcome::ToolCalls),
                    Some("max_tokens" | "model_context_window_exceeded") => {
                        Err(AgentMessageError::Length)
                    }
                    Some("refusal") => Err(AgentMessageError::ContentFilter),
                    Some(reason) => Err(other(format!("the response stopped: {reason}"))),
                    None => Err(other("the response stopped without a stop reason")),
                };
            }
            Event::Error { error } => {
                return Err(match error.r#type.as_str() {
                    "overloaded_error" => AgentMessageError::InsufficientSystemResource,
                    r#type => other(format!("{type}: {}", error.message)),
                });
            }
            Event::Other => {}
        }
    }

    Err(other("the response stream ended before message_stop"))
}

/// Append `text` to the string field `key` of a JSON object, creating the field if needed.
fn append(block: &mut Value, key: &str, text: &str) {
    match block.get_mut(key) {
        Some(Value::String(value)) => value.push_str(text),
        _ => block[key] = Value::String(text.to_owned()),
    }
}

fn build_request<'a>(agent: &'a Agent, tools: &'a [AgentTool]) -> Request<'a> {
    let (thinking, effort) = match agent.reasoning {
        Reasoning::Disabled => (Some(Thinking::Disabled), None),
        Reasoning::BetweenTools => (Some(Thinking::BetweenTools), None),
        // Thinking is hidden by default; summaries keep reasoning visible in the stream.
        Reasoning::Enabled(effort) => (
            Some(Thinking::Adaptive {
                display: "summarized",
            }),
            effort,
        ),
    };

    // Leading system messages form the system prompt; later ones go in the conversation.
    let leading_system = agent
        .dialog
        .iter()
        .take_while(|message| matches!(message, DialogMessage::System { .. }))
        .count();
    let system: Vec<&str> = agent.dialog[..leading_system]
        .iter()
        .filter_map(|message| match message {
            DialogMessage::System { content } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    let mut messages = Vec::<Message>::new();
    for message in &agent.dialog[leading_system..] {
        match message {
            DialogMessage::System { content } => messages.push(Message {
                role: "system",
                content: Content::Text(content),
            }),
            DialogMessage::User { content } => {
                push_blocks(&mut messages, "user", vec![Block::Text { text: content }]);
            }
            DialogMessage::Assistant {
                content,
                tool_calls,
                raw_output,
                ..
            } => {
                let raw = raw_output.items(ApiDialect::Messages);
                let blocks = if raw.is_empty() {
                    // Reasoning without a signature cannot be sent back.
                    let text = (!content.is_empty()).then_some(Block::Text { text: content });
                    let calls = tool_calls.iter().map(|tool_call| Block::ToolUse {
                        id: &tool_call.id,
                        name: &tool_call.name,
                        input: serde_json::from_str(&tool_call.arguments)
                            .unwrap_or_else(|_| Value::Object(Default::default())),
                    });
                    text.into_iter().chain(calls).collect()
                } else {
                    raw.iter().map(Block::Raw).collect()
                };
                push_blocks(&mut messages, "assistant", blocks);
            }
            // Consecutive results are merged into one message, as parallel calls need.
            DialogMessage::Tool { id, result } => push_blocks(
                &mut messages,
                "user",
                vec![Block::ToolResult {
                    tool_use_id: id,
                    content: result,
                }],
            ),
        }
    }

    Request {
        model: &agent.model,
        max_tokens: MAX_TOKENS,
        system: (!system.is_empty()).then(|| system.join("\n\n")),
        messages,
        tools: tools
            .iter()
            .map(|tool| Tool {
                name: &tool.name,
                description: &tool.description,
                input_schema: &tool.parameters,
                eager_input_streaming: true,
            })
            .collect(),
        thinking,
        output_config: effort.map(|effort| OutputConfig {
            effort: effort.as_str(),
        }),
        stream: true,
    }
}

/// Add blocks to the conversation, merging them into the last message if it has the same role.
fn push_blocks<'a>(messages: &mut Vec<Message<'a>>, role: &'static str, blocks: Vec<Block<'a>>) {
    if blocks.is_empty() {
        return;
    }
    if let Some(Message {
        role: last_role,
        content: Content::Blocks(last_blocks),
    }) = messages.last_mut()
        && *last_role == role
    {
        last_blocks.extend(blocks);
        return;
    }
    messages.push(Message {
        role,
        content: Content::Blocks(blocks),
    });
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<Thinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: Content<'a>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Content<'a> {
    Text(&'a str),
    Blocks(Vec<Block<'a>>),
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block<'a> {
    Text {
        text: &'a str,
    },
    ToolUse {
        id: &'a str,
        name: &'a str,
        input: Value,
    },
    ToolResult {
        tool_use_id: &'a str,
        content: &'a str,
    },
    /// A content block sent back as it was received.
    #[serde(untagged)]
    Raw(&'a Value),
}

#[derive(Serialize)]
struct Tool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a Value,
    eager_input_streaming: bool,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Thinking {
    Disabled,
    BetweenTools,
    Adaptive { display: &'static str },
}

#[derive(Serialize)]
struct OutputConfig {
    effort: &'static str,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    ContentBlockStart {
        index: usize,
        content_block: Value,
    },
    ContentBlockDelta {
        index: usize,
        delta: Delta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: MessageDelta,
    },
    MessageStop,
    Error {
        error: ErrorBody,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[expect(
    clippy::enum_variant_names,
    reason = "the variants are named after the delta types"
)]
enum Delta {
    TextDelta {
        text: String,
    },
    ThinkingDelta {
        thinking: String,
    },
    SignatureDelta {
        signature: String,
    },
    InputJsonDelta {
        partial_json: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct MessageDelta {
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ErrorBody {
    r#type: String,
    message: String,
}
