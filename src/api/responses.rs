use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, DialogMessage},
    api::{EventSender, StreamOutcome, emit, other, post_sse},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::Reasoning as AgentReasoning,
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
                    return Err(AgentMessageError::Other(
                        "function call output item is missing required fields".to_owned(),
                    ));
                };
                let arguments = item.arguments.unwrap_or_default();
                tool_calls.insert(item_id, (call_id.clone(), arguments.clone()));
                emit(
                    tx,
                    AgentMessageDelta::ToolCall {
                        name,
                        arguments,
                        tool_call_id: call_id,
                    },
                )?;
            }
            Event::FunctionCallArgumentsDelta { item_id, delta } => {
                let Some((call_id, arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(AgentMessageError::Other(format!(
                        "received arguments for unknown function call item: {item_id}"
                    )));
                };
                arguments.push_str(&delta);
                emit(
                    tx,
                    AgentMessageDelta::ToolCall {
                        name: String::new(),
                        arguments: delta,
                        tool_call_id: call_id.clone(),
                    },
                )?;
            }
            Event::FunctionCallArgumentsDone { item_id, arguments } => {
                let Some((call_id, streamed_arguments)) = tool_calls.get_mut(&item_id) else {
                    return Err(AgentMessageError::Other(format!(
                        "received completed arguments for unknown function call item: {item_id}"
                    )));
                };
                if let Some(remaining) = arguments.strip_prefix(streamed_arguments.as_str()) {
                    if !remaining.is_empty() {
                        emit(
                            tx,
                            AgentMessageDelta::ToolCall {
                                name: String::new(),
                                arguments: remaining.to_owned(),
                                tool_call_id: call_id.clone(),
                            },
                        )?;
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

fn build_request<'a>(agent: &'a Agent, tools: &'a [AgentTool]) -> Request<'a> {
    let effort = match agent.reasoning {
        AgentReasoning::Disabled => Some("none"),
        AgentReasoning::Enabled(effort) => effort.map(|effort| effort.as_str()),
    };

    Request {
        model: &agent.model,
        input: agent.dialog.iter().flat_map(input_items).collect(),
        reasoning: effort.map(|effort| Reasoning { effort }),
        text: Text {
            format: TextFormat { r#type: "text" },
        },
        stream: true,
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

fn input_items(message: &DialogMessage) -> Vec<InputItem<'_>> {
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
        } => {
            let reasoning = (!reasoning_content.is_empty()).then(|| InputItem::Reasoning {
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
            reasoning.into_iter().chain(message).chain(calls).collect()
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
