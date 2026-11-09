use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, ApiDialect, DialogMessage},
    api::{EventSender, StreamEvent, StreamOutcome, emit, other, post_sse, send},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::{Reasoning as AgentReasoning, ReasoningFormat},
    tool::Tool as AgentTool,
};

pub(super) fn stream_response(
    agent: &Agent,
    tools: &[AgentTool],
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut tool_calls = HashMap::<String, (String, String)>::new();

    for data in post_sse(agent, "/responses", &build_request(agent, tools))? {
        let event: Event = serde_json::from_str(&data?).map_err(other)?;
        match event {
            Event::ReasoningTextDelta { delta } => {
                emit(tx, AgentMessageDelta::ReasoningContent(delta))?;
            }
            Event::OutputTextDelta { delta } => {
                emit(tx, AgentMessageDelta::Content(delta))?;
            }
            Event::OutputItemAdded { item } if item.r#type == "function_call" => {
                let (Some(item_id), Some(call_id), Some(name)) = (item.id, item.call_id, item.name)
                else {
                    return Err(other(
                        "function call output item is missing required fields",
                    ));
                };
                let arguments = item.arguments.unwrap_or_default();
                tool_calls.insert(item_id, (call_id.clone(), arguments.clone()));
                emit(
                    tx,
                    AgentMessageDelta::ToolCallStart {
                        id: call_id.clone(),
                        name,
                    },
                )?;
                if !arguments.is_empty() {
                    emit(
                        tx,
                        AgentMessageDelta::ToolCallArguments {
                            id: call_id,
                            arguments,
                        },
                    )?;
                }
            }
            Event::FunctionCallArgumentsDelta { item_id, delta } => {
                let Some((call_id, arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(other(format!(
                        "received arguments for unknown function call item: {item_id}"
                    )));
                };
                arguments.push_str(&delta);
                emit(
                    tx,
                    AgentMessageDelta::ToolCallArguments {
                        id: call_id.clone(),
                        arguments: delta,
                    },
                )?;
            }
            Event::FunctionCallArgumentsDone { item_id, arguments } => {
                let Some((call_id, streamed_arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(other(format!(
                        "received completed arguments for unknown function call item: {item_id}"
                    )));
                };
                let Some(remaining) = arguments.strip_prefix(streamed_arguments.as_str()) else {
                    return Err(other(format!(
                        "streamed arguments do not match completed function call: {item_id}"
                    )));
                };
                if !remaining.is_empty() {
                    emit(
                        tx,
                        AgentMessageDelta::ToolCallArguments {
                            id: call_id.clone(),
                            arguments: remaining.to_owned(),
                        },
                    )?;
                }
                *streamed_arguments = arguments;
            }
            // Encrypted reasoning has to be sent back with the tool call outputs.
            Event::OutputItemDone { item }
                if item["type"] == "reasoning" && !item["encrypted_content"].is_null() =>
            {
                send(tx, StreamEvent::RawOutput(ApiDialect::Responses, item))?;
            }
            Event::Error { code, message } => {
                return Err(AgentMessageError::Other(match code {
                    Some(code) => format!("{code}: {message}"),
                    None => message,
                }));
            }
            Event::Completed { response } => {
                return if response
                    .output
                    .iter()
                    .any(|item| item.r#type == "function_call")
                {
                    Ok(StreamOutcome::ToolCalls)
                } else {
                    Ok(StreamOutcome::Finished)
                };
            }
            Event::Incomplete { response } => {
                let reason = response
                    .incomplete_details
                    .and_then(|details| details.reason);
                return match reason.as_deref() {
                    Some("max_output_tokens") => Err(AgentMessageError::Length),
                    Some("content_filter") => Err(AgentMessageError::ContentFilter),
                    Some(reason) => Err(AgentMessageError::Other(format!(
                        "response incomplete: {reason}"
                    ))),
                    None => Err(AgentMessageError::Other("response incomplete".to_owned())),
                };
            }
            Event::Failed { response } => {
                let message = response
                    .error
                    .map(|error| format!("{}: {}", error.code, error.message))
                    .unwrap_or_else(|| "response failed".to_owned());
                return Err(AgentMessageError::Other(message));
            }
            _ => {}
        }
    }

    Err(AgentMessageError::Other(
        "response stream ended without a terminal event".to_owned(),
    ))
}

fn build_request<'a>(agent: &'a Agent, tools: &'a [AgentTool]) -> Request<'a> {
    let effort = match agent.reasoning {
        AgentReasoning::Disabled | AgentReasoning::BetweenTools => Some("none"),
        AgentReasoning::Enabled(effort) => effort.map(|effort| effort.as_str()),
    };

    let encrypted = agent.reasoning_format == ReasoningFormat::Encrypted;
    Request {
        model: &agent.model,
        input: agent
            .dialog
            .iter()
            .flat_map(|message| input_items(message, agent.reasoning_format))
            .collect(),
        reasoning: effort.map(|effort| Reasoning { effort }),
        text: Text {
            format: TextFormat { r#type: "text" },
        },
        stream: true,
        // Keep requests stateless; reasoning comes back encrypted instead.
        store: encrypted.then_some(false),
        include: if encrypted {
            &["reasoning.encrypted_content"]
        } else {
            &[]
        },
        tools: tools
            .iter()
            .map(|tool| Tool {
                r#type: "function",
                description: &tool.description,
                name: &tool.name,
                parameters: &tool.parameters,
            })
            .collect(),
    }
}

fn input_items(message: &DialogMessage, reasoning_format: ReasoningFormat) -> Vec<InputItem<'_>> {
    match message {
        DialogMessage::System { content } => vec![InputItem::Message {
            role: "system",
            content,
        }],
        DialogMessage::User { content } => vec![InputItem::Message {
            role: "user",
            content,
        }],
        DialogMessage::Assistant {
            content,
            reasoning_content,
            tool_calls,
            raw_output,
        } => {
            let raw = raw_output.items(ApiDialect::Responses);
            let encrypted = raw.iter().map(InputItem::Raw);
            let plain = (raw.is_empty()
                && reasoning_format == ReasoningFormat::Plaintext
                && !reasoning_content.is_empty())
            .then(|| InputItem::Reasoning {
                content: [ReasoningContent {
                    r#type: "reasoning_text",
                    text: reasoning_content,
                }],
            });
            let message = (!content.is_empty()).then_some(InputItem::Message {
                role: "assistant",
                content,
            });
            let calls = tool_calls.iter().map(|tool_call| InputItem::FunctionCall {
                call_id: &tool_call.id,
                name: &tool_call.name,
                arguments: &tool_call.arguments,
            });
            encrypted.chain(plain).chain(message).chain(calls).collect()
        }
        DialogMessage::Tool { id, result } => vec![InputItem::FunctionCallOutput {
            call_id: id,
            output: result,
        }],
    }
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    input: Vec<InputItem<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Reasoning>,
    text: Text,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    include: &'static [&'static str],
    tools: Vec<Tool<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InputItem<'a> {
    Message {
        role: &'static str,
        content: &'a str,
    },
    Reasoning {
        content: [ReasoningContent<'a>; 1],
    },
    FunctionCall {
        call_id: &'a str,
        name: &'a str,
        arguments: &'a str,
    },
    FunctionCallOutput {
        call_id: &'a str,
        output: &'a str,
    },
    /// An output item sent back as it was received.
    #[serde(untagged)]
    Raw(&'a serde_json::Value),
}

#[derive(Serialize)]
struct ReasoningContent<'a> {
    r#type: &'static str,
    text: &'a str,
}

#[derive(Serialize)]
struct Reasoning {
    effort: &'static str,
}

#[derive(Serialize)]
struct Text {
    format: TextFormat,
}

#[derive(Serialize)]
struct TextFormat {
    r#type: &'static str,
}

#[derive(Serialize)]
struct Tool<'a> {
    r#type: &'static str,
    description: &'a str,
    name: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta { delta: String },
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta { delta: String },
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded { item: OutputItem },
    #[serde(rename = "response.output_item.done")]
    OutputItemDone { item: serde_json::Value },
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta { item_id: String, delta: String },
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone { item_id: String, arguments: String },
    #[serde(rename = "response.completed")]
    Completed { response: Response },
    #[serde(rename = "response.incomplete")]
    Incomplete { response: Response },
    #[serde(rename = "response.failed")]
    Failed { response: Response },
    #[serde(rename = "error")]
    Error {
        code: Option<String>,
        message: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    output: Vec<OutputItem>,
    #[serde(default)]
    incomplete_details: Option<IncompleteDetails>,
    #[serde(default)]
    error: Option<ResponseError>,
}

#[derive(Deserialize)]
struct OutputItem {
    r#type: String,
    id: Option<String>,
    call_id: Option<String>,
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct IncompleteDetails {
    reason: Option<String>,
}

#[derive(Deserialize)]
struct ResponseError {
    code: String,
    message: String,
}
