use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, DialogMessage},
    api::{EventSender, StreamOutcome, emit, other, post_sse},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    provider::{Reasoning, ReasoningEffort, ReasoningFormat},
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
            let id = match &tool_call_ids[index] {
                Some(id) => id.clone(),
                None => {
                    let (Some(id), Some(name)) = (tool_call.id, tool_call.function.name) else {
                        return Err(other("a tool call started without an id or a name"));
                    };
                    tool_call_ids[index] = Some(id.clone());
                    emit(
                        tx,
                        AgentMessageDelta::ToolCallStart {
                            id: id.clone(),
                            name,
                        },
                    )?;
                    id
                }
            };
            let arguments = tool_call.function.arguments;
            if !arguments.is_empty() {
                emit(tx, AgentMessageDelta::ToolCallArguments { id, arguments })?;
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

fn build_request<'a>(agent: &'a Agent, tools: &'a [Tool]) -> Request<'a> {
    let effort = |effort: Option<ReasoningEffort>| effort.map(ReasoningEffort::as_str);
    let (thinking, reasoning_effort) = match (agent.reasoning_format, agent.reasoning) {
        (ReasoningFormat::Plaintext, Reasoning::Disabled | Reasoning::BetweenTools) => {
            (Some("disabled"), None)
        }
        (ReasoningFormat::Plaintext, Reasoning::Enabled(level)) => (Some("enabled"), effort(level)),
        (ReasoningFormat::Encrypted, Reasoning::Disabled | Reasoning::BetweenTools) => {
            (None, Some("none"))
        }
        (ReasoningFormat::Encrypted, Reasoning::Enabled(level)) => (None, effort(level)),
    };

    Request {
        messages: agent
            .dialog
            .iter()
            .map(|message| Message::new(message, agent.reasoning_format))
            .collect(),
        model: &agent.model,
        thinking: thinking.map(|r#type| Thinking { r#type }),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<Thinking>,
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

impl<'a> Message<'a> {
    fn new(message: &'a DialogMessage, reasoning_format: ReasoningFormat) -> Self {
        match message {
            DialogMessage::System { content } => Self::System { content },
            DialogMessage::User { content } => Self::User { content },
            DialogMessage::Assistant {
                content,
                reasoning_content,
                tool_calls,
                ..
            } => Self::Assistant {
                content,
                // Chat Completions has no way to send encrypted reasoning back.
                reasoning_content: match reasoning_format {
                    ReasoningFormat::Plaintext => reasoning_content,
                    ReasoningFormat::Encrypted => "",
                },
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
