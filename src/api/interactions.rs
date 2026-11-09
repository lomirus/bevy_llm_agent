//! The Gemini Interactions API, used statelessly: every request sends the whole dialog.

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

pub(super) fn stream_response(
    agent: &Agent,
    tools: &[AgentTool],
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let headers = [("x-goog-api-key", agent.api_key.as_str())];
    let events = post_sse_with_headers(
        agent,
        "/interactions",
        &headers,
        &build_request(agent, tools),
    )?;
    read_events(events, tx)
}

/// A step being streamed, with its text or function call arguments so far.
struct OpenStep {
    step: Value,
    text: String,
    arguments: String,
}

fn read_events(
    events: impl Iterator<Item = Result<String, AgentMessageError>>,
    tx: &EventSender,
) -> Result<StreamOutcome, AgentMessageError> {
    let mut open_steps = HashMap::<usize, OpenStep>::new();

    for data in events {
        let data = data?;
        if data == "[DONE]" {
            break;
        }
        let event: Event = serde_json::from_str(&data).map_err(other)?;
        match event {
            Event::StepStart { index, step } => {
                if !step.is_object() {
                    return Err(other(format!("step {index} is not an object")));
                }
                if step["type"] == "function_call" {
                    let (Some(id), Some(name)) = (step["id"].as_str(), step["name"].as_str())
                    else {
                        return Err(other("a function call started without an id or a name"));
                    };
                    emit(
                        tx,
                        AgentMessageDelta::ToolCallStart {
                            id: id.to_owned(),
                            name: name.to_owned(),
                        },
                    )?;
                }
                open_steps.insert(
                    index,
                    OpenStep {
                        step,
                        text: String::new(),
                        arguments: String::new(),
                    },
                );
            }
            Event::StepDelta { index, delta } => {
                let Some(open) = open_steps.get_mut(&index) else {
                    return Err(other(format!("received a delta for unknown step {index}")));
                };
                match delta {
                    Delta::Text { text } => {
                        open.text.push_str(&text);
                        emit(tx, AgentMessageDelta::Content(text))?;
                    }
                    Delta::ThoughtSummary { content } => {
                        if let Some(text) = content["text"].as_str().filter(|text| !text.is_empty())
                        {
                            emit(tx, AgentMessageDelta::ReasoningContent(text.to_owned()))?;
                        }
                        match open.step.get_mut("summary") {
                            Some(Value::Array(summary)) => summary.push(content),
                            _ => open.step["summary"] = Value::Array(vec![content]),
                        }
                    }
                    Delta::ThoughtSignature { signature } => {
                        open.step["signature"] = Value::String(signature);
                    }
                    Delta::Arguments { arguments } => {
                        open.arguments.push_str(&arguments);
                        if !arguments.is_empty() {
                            let id = open.step["id"].as_str().unwrap_or_default().to_owned();
                            emit(tx, AgentMessageDelta::ToolCallArguments { id, arguments })?;
                        }
                    }
                    Delta::Other => {}
                }
            }
            Event::StepStop { index } => {
                let Some(OpenStep {
                    mut step,
                    text,
                    arguments,
                }) = open_steps.remove(&index)
                else {
                    return Err(other(format!("received the end of unknown step {index}")));
                };
                if !text.is_empty() {
                    step["content"] = serde_json::json!([{"type": "text", "text": text}]);
                }
                if !arguments.trim().is_empty() {
                    // Arguments that do not parse are still run, so the tool reports them to
                    // the model; the call is sent back without arguments.
                    step["arguments"] = serde_json::from_str(&arguments)
                        .unwrap_or_else(|_| Value::Object(Default::default()));
                }
                // Thought signatures have to be sent back with the steps around them.
                send(tx, StreamEvent::RawOutput(ApiDialect::Interactions, step))?;
            }
            Event::Completed { interaction } => {
                return match interaction.status.as_str() {
                    "completed" => Ok(StreamOutcome::Finished),
                    "requires_action" => Ok(StreamOutcome::ToolCalls),
                    "incomplete" => Err(AgentMessageError::Length),
                    status => Err(other(match interaction.errors {
                        Some(errors) => format!("interaction {status}: {errors}"),
                        None => format!("interaction {status}"),
                    })),
                };
            }
            Event::Error { error } => {
                return Err(other(match error.code {
                    Some(code) => format!("{code}: {}", error.message),
                    None => error.message,
                }));
            }
            Event::Other => {}
        }
    }

    Err(other("the interaction stream ended before it completed"))
}

fn build_request<'a>(agent: &'a Agent, tools: &'a [AgentTool]) -> Request<'a> {
    let (thinking_level, thinking_summaries) = match agent.reasoning {
        Reasoning::Disabled | Reasoning::BetweenTools => (None, None),
        Reasoning::Enabled(level) => (level.map(|level| level.as_str()), Some("auto")),
    };

    let mut system = Vec::new();
    let mut input = Vec::new();
    // Function results name the function they answer.
    let mut function_names = HashMap::<&str, &str>::new();
    for message in &agent.dialog {
        match message {
            DialogMessage::System { content } => system.push(content.as_str()),
            DialogMessage::User { content } => input.push(Step::UserInput {
                content: [TextContent::new(content)],
            }),
            DialogMessage::Assistant {
                content,
                tool_calls,
                raw_output,
                ..
            } => {
                for tool_call in tool_calls {
                    function_names.insert(&tool_call.id, &tool_call.name);
                }
                let raw = raw_output.items(ApiDialect::Interactions);
                if raw.is_empty() {
                    // Thoughts without a signature cannot be sent back.
                    if !content.is_empty() {
                        input.push(Step::ModelOutput {
                            content: [TextContent::new(content)],
                        });
                    }
                    input.extend(tool_calls.iter().map(|tool_call| {
                        Step::FunctionCall {
                            id: &tool_call.id,
                            name: &tool_call.name,
                            arguments: serde_json::from_str(&tool_call.arguments)
                                .unwrap_or_else(|_| Value::Object(Default::default())),
                        }
                    }));
                } else {
                    input.extend(raw.iter().map(Step::Raw));
                }
            }
            DialogMessage::Tool { id, result } => input.push(Step::FunctionResult {
                call_id: id,
                name: function_names.get(id.as_str()).copied(),
                result,
            }),
        }
    }

    Request {
        model: &agent.model,
        input,
        system_instruction: (!system.is_empty()).then(|| system.join("\n\n")),
        tools: tools
            .iter()
            .map(|tool| Tool {
                r#type: "function",
                name: &tool.name,
                description: &tool.description,
                parameters: &tool.parameters,
            })
            .collect(),
        generation_config: GenerationConfig {
            thinking_level,
            thinking_summaries,
        },
        stream: true,
        store: false,
    }
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    input: Vec<Step<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Tool<'a>>,
    generation_config: GenerationConfig,
    stream: bool,
    store: bool,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Step<'a> {
    UserInput {
        content: [TextContent<'a>; 1],
    },
    ModelOutput {
        content: [TextContent<'a>; 1],
    },
    FunctionCall {
        id: &'a str,
        name: &'a str,
        arguments: Value,
    },
    FunctionResult {
        call_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<&'a str>,
        result: &'a str,
    },
    /// A step sent back as it was received.
    #[serde(untagged)]
    Raw(&'a Value),
}

#[derive(Serialize)]
struct TextContent<'a> {
    r#type: &'static str,
    text: &'a str,
}

impl<'a> TextContent<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            r#type: "text",
            text,
        }
    }
}

#[derive(Serialize)]
struct Tool<'a> {
    r#type: &'static str,
    name: &'a str,
    description: &'a str,
    parameters: &'a Value,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_level: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_summaries: Option<&'static str>,
}

#[derive(Deserialize)]
#[serde(tag = "event_type")]
enum Event {
    #[serde(rename = "step.start")]
    StepStart { index: usize, step: Value },
    #[serde(rename = "step.delta")]
    StepDelta { index: usize, delta: Delta },
    #[serde(rename = "step.stop")]
    StepStop { index: usize },
    #[serde(rename = "interaction.completed")]
    Completed { interaction: Interaction },
    #[serde(rename = "error")]
    Error { error: ErrorBody },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Delta {
    Text {
        text: String,
    },
    ThoughtSummary {
        content: Value,
    },
    ThoughtSignature {
        signature: String,
    },
    #[serde(rename = "arguments_delta")]
    Arguments {
        arguments: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Interaction {
    status: String,
    #[serde(default)]
    errors: Option<Value>,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    code: Option<String>,
    message: String,
}
