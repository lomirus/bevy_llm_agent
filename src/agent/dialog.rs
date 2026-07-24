use crate::{
    agent::{DialogMessage, ToolCall},
    messages::agent_message::AgentMessageDelta,
};

pub(crate) fn apply_delta(dialog: &mut Vec<DialogMessage>, delta: &AgentMessageDelta) {
    match delta {
        AgentMessageDelta::Content(text) => match dialog.last_mut() {
            Some(DialogMessage::Assistant { content, .. }) => {
                content.push_str(text);
            }
            _ => {
                dialog.push(DialogMessage::Assistant {
                    content: text.clone(),
                    reasoning_content: String::new(),
                    tool_calls: Vec::new(),
                });
            }
        },
        AgentMessageDelta::ReasoningContent(text) => match dialog.last_mut() {
            Some(DialogMessage::Assistant {
                reasoning_content, ..
            }) => {
                reasoning_content.push_str(text);
            }
            _ => {
                dialog.push(DialogMessage::Assistant {
                    content: String::new(),
                    reasoning_content: text.clone(),
                    tool_calls: Vec::new(),
                });
            }
        },
        AgentMessageDelta::ToolCall {
            name,
            arguments,
            tool_call_id,
        } => {
            if tool_call_id.is_empty() {
                assert!(name.is_empty());
                let DialogMessage::Assistant { tool_calls, .. } = dialog.last_mut().unwrap() else {
                    unreachable!()
                };
                tool_calls.last_mut().unwrap().arguments.push_str(arguments);
            } else {
                match dialog.last_mut() {
                    Some(DialogMessage::Assistant { tool_calls, .. }) => {
                        tool_calls.push(ToolCall {
                            id: tool_call_id.clone(),
                            name: name.clone(),
                            arguments: arguments.clone(),
                        });
                    }
                    _ => {
                        dialog.push(DialogMessage::Assistant {
                            content: String::new(),
                            reasoning_content: String::new(),
                            tool_calls: vec![ToolCall {
                                id: tool_call_id.clone(),
                                name: name.clone(),
                                arguments: arguments.clone(),
                            }],
                        });
                    }
                }
            }
        }
        AgentMessageDelta::ToolResult {
            content,
            tool_call_id,
        } => {
            dialog.push(DialogMessage::Tool {
                id: tool_call_id.clone(),
                result: content.clone(),
            });
        }
        AgentMessageDelta::Finish(_) => unreachable!(),
    }
}
