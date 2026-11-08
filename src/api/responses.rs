use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    sync::mpsc,
};

use serde::{Deserialize, Serialize};

use crate::{
    agent::DialogMessage,
    api::{DeltaResult, StreamOutcome, emit_delta},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::Reasoning as AgentReasoning,
    tool::Tool as AgentTool,
};

pub(super) fn stream_response(
    dialog: &mut Vec<DialogMessage>,
    model: &str,
    reasoning: AgentReasoning,
    tools: &[AgentTool],
    api_base_url: &str,
    api_key: &str,
    tx: &mpsc::Sender<DeltaResult>,
) -> Result<StreamOutcome, AgentMessageError> {
    let body = build_request(dialog, model, reasoning, tools);
    let url = format!("{api_base_url}/responses");
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
    reasoning: AgentReasoning,
    tools: &[AgentTool],
) -> Request {
    let effort = match reasoning {
        AgentReasoning::Disabled => Some("none"),
        AgentReasoning::Enabled(effort) => effort.map(|effort| effort.as_str()),
    };

    Request {
        model: model.to_owned(),
        input: dialog_to_input(dialog),
        reasoning: effort.map(|effort| Reasoning { effort }),
        text: Text {
            format: TextFormat { r#type: "text" },
        },
        stream: true,
        tools: tools
            .iter()
            .map(|tool| Tool {
                r#type: "function",
                description: tool.description.clone(),
                name: tool.name.clone(),
                parameters: tool.parameters.clone(),
            })
            .collect(),
    }
}

fn dialog_to_input(dialog: &[DialogMessage]) -> Vec<InputItem> {
    let mut input = Vec::new();

    for message in dialog {
        match message {
            DialogMessage::System { content } => input.push(InputItem::Message {
                role: "system",
                content: content.clone(),
            }),
            DialogMessage::User { content } => input.push(InputItem::Message {
                role: "user",
                content: content.clone(),
            }),
            DialogMessage::Assistant {
                content,
                reasoning_content,
                tool_calls,
            } => {
                if !reasoning_content.is_empty() {
                    input.push(InputItem::Reasoning {
                        content: vec![ReasoningContent {
                            r#type: "reasoning_text",
                            text: reasoning_content.clone(),
                        }],
                    });
                }
                if !content.is_empty() {
                    input.push(InputItem::Message {
                        role: "assistant",
                        content: content.clone(),
                    });
                }
                input.extend(tool_calls.iter().map(|tool_call| InputItem::FunctionCall {
                    call_id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                }));
            }
            DialogMessage::Tool { id, result } => input.push(InputItem::FunctionCallOutput {
                call_id: id.clone(),
                output: result.clone(),
            }),
        }
    }

    input
}

fn consume_sse_stream(
    reader: impl BufRead,
    dialog: &mut Vec<DialogMessage>,
    tx: &mpsc::Sender<DeltaResult>,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut tool_calls = HashMap::<String, (String, String)>::new();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                return Err(AgentMessageError::Other(error.to_string()));
            }
        };
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        let event: Event = match serde_json::from_str(data) {
            Ok(event) => event,
            Err(error) => {
                return Err(AgentMessageError::Other(error.to_string()));
            }
        };

        match event {
            Event::ReasoningTextDelta { delta } => {
                emit_delta(dialog, tx, AgentMessageDelta::ReasoningContent(delta));
            }
            Event::OutputTextDelta { delta } => {
                emit_delta(dialog, tx, AgentMessageDelta::Content(delta));
            }
            Event::OutputItemAdded { item } if item.r#type == "function_call" => {
                let (Some(item_id), Some(call_id), Some(name)) = (item.id, item.call_id, item.name)
                else {
                    return Err(AgentMessageError::Other(
                        "function call output item is missing required fields".to_owned(),
                    ));
                };
                let arguments = item.arguments.unwrap_or_default();
                tool_calls.insert(item_id, (call_id.clone(), arguments.clone()));
                emit_delta(
                    dialog,
                    tx,
                    AgentMessageDelta::ToolCall {
                        name,
                        arguments,
                        tool_call_id: call_id,
                    },
                );
            }
            Event::FunctionCallArgumentsDelta { item_id, delta } => {
                let Some((call_id, arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(AgentMessageError::Other(format!(
                        "received arguments for unknown function call item: {item_id}"
                    )));
                };
                arguments.push_str(&delta);
                emit_delta(
                    dialog,
                    tx,
                    AgentMessageDelta::ToolCall {
                        name: String::new(),
                        arguments: delta,
                        tool_call_id: call_id.clone(),
                    },
                );
            }
            Event::FunctionCallArgumentsDone { item_id, arguments } => {
                let Some((call_id, streamed_arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(AgentMessageError::Other(format!(
                        "received completed arguments for unknown function call item: {item_id}"
                    )));
                };
                if let Some(remaining) = arguments.strip_prefix(streamed_arguments.as_str()) {
                    if !remaining.is_empty() {
                        emit_delta(
                            dialog,
                            tx,
                            AgentMessageDelta::ToolCall {
                                name: String::new(),
                                arguments: remaining.to_owned(),
                                tool_call_id: call_id.clone(),
                            },
                        );
                    }
                    *streamed_arguments = arguments;
                } else {
                    return Err(AgentMessageError::Other(format!(
                        "streamed arguments do not match completed function call: {item_id}"
                    )));
                }
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

#[derive(Serialize)]
struct Request {
    model: String,
    input: Vec<InputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<Reasoning>,
    text: Text,
    stream: bool,
    tools: Vec<Tool>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InputItem {
    Message {
        role: &'static str,
        content: String,
    },
    Reasoning {
        content: Vec<ReasoningContent>,
    },
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

#[derive(Serialize)]
struct ReasoningContent {
    r#type: &'static str,
    text: String,
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
struct Tool {
    r#type: &'static str,
    description: String,
    name: String,
    parameters: serde_json::Value,
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
