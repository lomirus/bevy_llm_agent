use crate::{
    AgentMessageDelta, FinishReason, UserMessage,
    agent::{Agent, AgentStatus, DialogMessage, Thinking, ToolCall},
    chat_completion::{self, ResponseFormat, ToolFunction},
    plugin::RawToolInvocationSender,
    tool::{AgentTools, RawToolInvocation, Tool},
};
use bevy::{ecs::query::QueryEntityError, prelude::*};
use std::{
    io::{BufRead, BufReader},
    sync::{Mutex, mpsc},
    thread,
};

enum StreamOutcome {
    Finished(FinishReason),
    ToolCalls {
        content: String,
        reasoning_content: String,
        tool_calls: Vec<ToolCall>,
    },
}

fn consume_sse_stream(reader: impl BufRead, tx: &mpsc::Sender<AgentMessageDelta>) -> StreamOutcome {
    let mut final_reasoning_content = String::new();
    let mut final_content = String::new();
    let mut final_tool_calls: Vec<ToolCall> = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        if data == "[DONE]" {
            unreachable!()
        }
        let data: chat_completion::Response = serde_json::from_str(data).unwrap();
        assert_eq!(data.choices.len(), 1);
        let choice = &data.choices[0];
        if let Some(reason) = &choice.finish_reason {
            use chat_completion::FinishReason::*;
            return match reason {
                Stop => StreamOutcome::Finished(FinishReason::Stop),
                Length => StreamOutcome::Finished(FinishReason::Length),
                ContentFilter => StreamOutcome::Finished(FinishReason::ContentFilter),
                InsufficientSystemResource => {
                    StreamOutcome::Finished(FinishReason::InsufficientSystemResource)
                }
                ToolCalls => StreamOutcome::ToolCalls {
                    content: final_content,
                    reasoning_content: final_reasoning_content,
                    tool_calls: final_tool_calls,
                },
            };
        }
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
            if let Some(tool_call_id) = tool_call.id {
                assert!(tool_call.function.name.is_some());
                final_tool_calls.push(ToolCall {
                    id: tool_call_id,
                    name: tool_call.function.name.unwrap(),
                    arguments: tool_call.function.arguments,
                });
            } else {
                assert!(tool_call.function.name.is_none());
                let last_tool_call = final_tool_calls.last_mut().unwrap();
                last_tool_call
                    .arguments
                    .push_str(&tool_call.function.arguments);
            }
        }
    }
    unreachable!()
}

fn run_agent_loop(
    dialog: &mut Vec<DialogMessage>,
    model: String,
    thinking: Thinking,
    tools: Vec<Tool>,
    api_key: String,
    raw_tool_invocation_sender: crossbeam::channel::Sender<RawToolInvocation>,
    tx: mpsc::Sender<AgentMessageDelta>,
) {
    loop {
        use crate::agent::Thinking::*;
        let body = chat_completion::Request {
            messages: dialog.iter().map(|msg| msg.clone().into()).collect(),
            model: model.clone(),
            thinking: chat_completion::Thinking {
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

        match consume_sse_stream(reader, &tx) {
            StreamOutcome::Finished(reason) => {
                tx.send(AgentMessageDelta::Finish(reason)).unwrap();
                return;
            }
            StreamOutcome::ToolCalls {
                content,
                reasoning_content,
                tool_calls,
            } => {
                dialog.push(DialogMessage::Assistant {
                    content,
                    reasoning_content,
                    tool_calls: tool_calls.clone(),
                });

                let mut tool_call_waiters = Vec::with_capacity(tool_calls.len());

                for tool_call in tool_calls {
                    let (tx, rx) = oneshot::channel();
                    tool_call_waiters.push((tool_call.id, rx));

                    raw_tool_invocation_sender
                        .send(RawToolInvocation {
                            raw_args: tool_call.arguments,
                            raw_responder: tx,
                            dispatch: tools
                                .iter()
                                .find(|tool| tool.name == tool_call.name)
                                .unwrap()
                                .dispatch,
                        })
                        .unwrap();
                }

                for tool_call_waiter in tool_call_waiters {
                    let result = tool_call_waiter.1.recv().unwrap();
                    let tool_call_id = tool_call_waiter.0;
                    dialog.push(DialogMessage::Tool {
                        id: tool_call_id.clone(),
                        result: result.clone(),
                    });
                    tx.send(AgentMessageDelta::ToolResult {
                        content: result,
                        tool_call_id,
                    })
                    .unwrap();
                }
            }
        }
    }
}

pub(crate) fn read_user_message(
    mut agent_query: Query<&mut Agent>,
    agent_tools_query: Query<&AgentTools>,
    tool_query: Query<&Tool>,
    mut reader: MessageReader<UserMessage>,
    raw_tool_invocation_sender: Res<RawToolInvocationSender>,
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

        let tools: Vec<Tool> = match agent_tools_query.get(*entity) {
            Ok(tools) => tools
                .iter()
                .map(|entity| tool_query.get(entity).unwrap().clone())
                .collect(),
            Err(QueryEntityError::QueryDoesNotMatch(..)) => Vec::new(),
            Err(err) => panic!("{err}"),
        };

        let api_key = agent.api_key.to_owned();
        let model = agent.model.to_owned();
        let thinking = agent.thinking.to_owned();
        let dialog = agent.dialog.to_owned();
        let raw_tool_invocation_sender = raw_tool_invocation_sender.clone();

        thread::spawn(move || {
            let mut dialog = dialog;
            run_agent_loop(
                &mut dialog,
                model,
                thinking,
                tools,
                api_key,
                raw_tool_invocation_sender,
                tx,
            );
        });
    }
}
