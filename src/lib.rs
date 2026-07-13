pub mod agent;
mod chat_completion;
pub mod tool;

use std::{
    io::{BufRead, BufReader},
    sync::{Mutex, mpsc},
    thread,
};

use agent::Agent;
use bevy::prelude::*;

use crate::{
    agent::{AgentStatus, DialogMessage, ToolCall},
    chat_completion::{ResponseFormat, Thinking, ToolFunction},
    tool::{AgentTools, Tool},
};

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: AgentMessageDelta,
}

pub enum AgentMessageDelta {
    Content(String),
    ReasoningContent(String),
    ToolCall {
        name: String,
        arguments: String,
        tool_call_id: String,
    },
    ToolResult {
        content: String,
        tool_call_id: String,
    },
    Finish(FinishReason),
}

pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    InsufficientSystemResource,
}

#[derive(Message)]
pub struct UserMessage {
    pub entity: Entity,
    pub prompt: String,
}

impl UserMessage {
    pub fn new(entity: Entity, prompt: impl Into<String>) -> Self {
        UserMessage {
            entity,
            prompt: prompt.into(),
        }
    }
}

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<UserMessage>()
            .add_message::<AgentMessage>()
            .add_systems(FixedUpdate, (read_user_message, write_agent_message));
    }
}

pub const DEEPSEEK_V4_FLASH: &str = "deepseek-v4-flash";
pub const DEEPSEEK_V4_PRO: &str = "deepseek-v4-pro";

fn read_user_message(
    mut agent_query: Query<&mut Agent>,
    agent_tools_query: Query<&AgentTools>,
    tool_query: Query<&Tool>,
    mut reader: MessageReader<UserMessage>,
) {
    for UserMessage { entity, prompt } in reader.read() {
        let mut agent = agent_query.get_mut(*entity).unwrap();

        agent.dialog.push(DialogMessage::User {
            content: prompt.to_owned(),
        });

        let (tx, rx) = mpsc::channel::<AgentMessageDelta>();
        if let AgentStatus::Streaming(_) = agent.status {
            unreachable!();
        }
        agent.status = AgentStatus::Streaming(Mutex::new(rx));

        let tools: Vec<_> = agent_tools_query
            .get(*entity)
            .unwrap()
            .iter()
            .map(|entity| tool_query.get(entity).unwrap().clone())
            .collect();

        let api_key = agent.api_key.to_owned();
        let model = agent.model.to_owned();
        let thinking = agent.thinking.to_owned();
        let dialog = agent.dialog.to_owned();

        thread::spawn(move || {
            let mut dialog = dialog;
            'outer: loop {
                use agent::Thinking::*;
                let body = chat_completion::Request {
                    messages: dialog.iter().map(|msg| msg.clone().into()).collect(),
                    model: model.clone(),
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
                        .map(|tool| crate::chat_completion::Tool {
                            r#type: "function".to_owned(),
                            function: ToolFunction {
                                description: tool.description.clone(),
                                name: tool.name.clone(),
                                parameters: tool.parameters.clone(),
                            },
                        })
                        .collect(),
                };

                let mut resp = ureq::post("https://api.deepseek.com/chat/completions")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send_json(body)
                    .unwrap();

                let reader = BufReader::new(resp.body_mut().as_reader());

                let mut final_reasoning_content = String::new();
                let mut final_content = String::new();
                let mut final_tool_calls = Vec::new();

                for line in reader.lines().map_while(Result::ok) {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            unreachable!()
                        }
                        let data: chat_completion::Response = serde_json::from_str(data).unwrap();
                        assert_eq!(data.choices.len(), 1);
                        let choice = &data.choices[0];
                        if let Some(reason) = &choice.finish_reason {
                            use chat_completion::FinishReason::*;
                            match reason {
                                Stop => {
                                    tx.send(AgentMessageDelta::Finish(FinishReason::Stop))
                                        .unwrap();
                                    break 'outer;
                                }
                                ToolCalls => {
                                    dialog.push(DialogMessage::Assistant {
                                        content: final_content,
                                        reasoning_content: final_reasoning_content,
                                        tool_calls: final_tool_calls,
                                    });
                                    // TODO: Dispatch the tool
                                    // PROBLEM: dispatch(world, ...) 时，world 会阻塞引擎且不能跨线程传输
                                    break;
                                }
                                Length => {
                                    tx.send(AgentMessageDelta::Finish(FinishReason::Length))
                                        .unwrap();
                                    break 'outer;
                                }
                                ContentFilter => {
                                    tx.send(AgentMessageDelta::Finish(FinishReason::ContentFilter))
                                        .unwrap();
                                    break 'outer;
                                }
                                InsufficientSystemResource => {
                                    tx.send(AgentMessageDelta::Finish(
                                        FinishReason::InsufficientSystemResource,
                                    ))
                                    .unwrap();
                                    break 'outer;
                                }
                            }
                        }
                        // TODO: Update the agent.dialog by sending a message.
                        if let Some(reasoning_content) = &choice.delta.reasoning_content {
                            assert!(choice.delta.content.is_none());
                            assert!(choice.delta.tool_calls.is_none());
                            tx.send(AgentMessageDelta::ReasoningContent(
                                reasoning_content.to_owned(),
                            ))
                            .unwrap();
                            final_reasoning_content.push_str(reasoning_content);
                        } else if let Some(content) = &choice.delta.content {
                            assert!(choice.delta.tool_calls.is_none());
                            tx.send(AgentMessageDelta::Content(content.to_owned()))
                                .unwrap();
                            final_content.push_str(content);
                        } else if let Some(tool_calls) = &choice.delta.tool_calls {
                            assert_eq!(tool_calls.len(), 1);
                            let tool_call = tool_calls[0].clone();
                            tx.send(AgentMessageDelta::ToolCall {
                                name: tool_call.function.name.clone().unwrap_or_default(),
                                arguments: tool_call.function.arguments.clone(),
                                tool_call_id: tool_call.id.clone().unwrap_or_default(),
                            })
                            .unwrap();
                            if final_tool_calls.is_empty() {
                                final_tool_calls.push(ToolCall {
                                    id: tool_call.id.unwrap(),
                                    name: tool_call.function.name.unwrap(),
                                    arguments: tool_call.function.arguments,
                                });
                            } else {
                                let last_tool_call = final_tool_calls.last_mut().unwrap();
                                assert!(tool_call.id.is_none());
                                assert!(tool_call.function.name.is_none());
                                last_tool_call
                                    .arguments
                                    .push_str(&tool_call.function.arguments);
                            }
                        }
                    }
                }
            }
        });
    }
}

fn write_agent_message(
    mut agents: Query<(Entity, &mut Agent)>,
    mut writer: MessageWriter<AgentMessage>,
) {
    for (entity, mut agent) in agents.iter_mut() {
        if let AgentStatus::Streaming(receiver) = &mut agent.status {
            let receiver = receiver.lock().unwrap();
            while let Ok(delta) = receiver.try_recv() {
                writer.write(AgentMessage { entity, delta });
            }
        }
    }
}
