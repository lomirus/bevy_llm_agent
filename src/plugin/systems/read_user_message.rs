use crate::{
    agent::{Agent, AgentStatus, ApiDialect, DialogMessage, Thinking},
    api::{self, DeltaResult, StreamOutcome},
    messages::{agent_message::AgentMessageDelta, user_message::UserMessage},
    plugin::RawToolCallSender,
    tool::{AgentTools, RawToolCall, Tool},
};
use bevy::{ecs::query::QueryEntityError, prelude::*};
use std::{
    sync::{Mutex, mpsc},
    thread,
};

fn run_agent_loop(
    dialog: &mut Vec<DialogMessage>,
    api_dialect: ApiDialect,
    model: String,
    thinking: Thinking,
    tools: Vec<Tool>,
    api_base_url: String,
    api_key: String,
    raw_tool_invocation_sender: crossbeam_channel::Sender<RawToolCall>,
    tx: mpsc::Sender<DeltaResult>,
    entity: Entity,
) {
    loop {
        match api::stream_response(
            api_dialect,
            dialog,
            &model,
            &thinking,
            &tools,
            &api_base_url,
            &api_key,
            &tx,
        ) {
            Ok(StreamOutcome::Finished) => {
                tx.send(Ok(AgentMessageDelta::Finish)).unwrap();
                return;
            }
            Err(err) => {
                tx.send(Err(err)).unwrap();
                return;
            }
            Ok(StreamOutcome::ToolCalls) => {
                let DialogMessage::Assistant { tool_calls, .. } = dialog.last().unwrap() else {
                    unreachable!()
                };
                let tool_calls = tool_calls.clone();
                let mut tool_call_waiters = Vec::with_capacity(tool_calls.len());

                for tool_call in tool_calls {
                    let (tx, rx) = oneshot::channel();
                    tool_call_waiters.push((tool_call.id, rx));

                    raw_tool_invocation_sender
                        .send(RawToolCall {
                            entity,
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
                    api::emit_delta(
                        dialog,
                        &tx,
                        AgentMessageDelta::ToolResult {
                            content: result,
                            tool_call_id,
                        },
                    );
                }
            }
        }
    }
}

pub(crate) fn read_user_message(
    mut agent_query: Query<(&mut Agent, &mut AgentStatus)>,
    agent_tools_query: Query<&AgentTools>,
    tool_query: Query<&Tool>,
    mut reader: MessageReader<UserMessage>,
    raw_tool_invocation_sender: Res<RawToolCallSender>,
) {
    for UserMessage { entity, prompt } in reader.read() {
        let (mut agent, mut agent_status) = agent_query.get_mut(*entity).unwrap();

        if let AgentStatus::Streaming(_) = *agent_status {
            warn!("ignoring user message; agent {entity} is already streaming");
            continue;
        }

        agent.dialog.push(DialogMessage::User {
            content: prompt.to_owned(),
        });

        let (tx, rx) = mpsc::channel::<DeltaResult>();
        *agent_status = AgentStatus::Streaming(Mutex::new(rx));

        let tools: Vec<Tool> = match agent_tools_query.get(*entity) {
            Ok(tools) => tools
                .iter()
                .map(|entity| tool_query.get(entity).unwrap().clone())
                .collect(),
            Err(QueryEntityError::QueryDoesNotMatch(..)) => Vec::new(),
            Err(err) => panic!("{err}"),
        };

        let api_dialect = agent.api_dialect;
        let model = agent.model.to_owned();
        let thinking = agent.thinking.to_owned();
        let dialog = agent.dialog.to_owned();
        let api_base_url = agent.api_base_url.to_owned();
        let api_key = agent.api_key.to_owned();
        let raw_tool_invocation_sender = raw_tool_invocation_sender.clone();
        let entity = entity.clone();

        thread::spawn(move || {
            let mut dialog = dialog;
            run_agent_loop(
                &mut dialog,
                api_dialect,
                model,
                thinking,
                tools,
                api_base_url,
                api_key,
                raw_tool_invocation_sender,
                tx,
                entity,
            );
        });
    }
}
