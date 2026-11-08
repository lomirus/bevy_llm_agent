use crate::{
    agent::{DialogMessage, EncryptedReasoning, ToolCall},
    messages::agent_message::AgentMessageDelta,
};

pub(crate) fn apply_delta(dialog: &mut Vec<DialogMessage>, delta: &AgentMessageDelta) {
    if let AgentMessageDelta::ToolResult {
        content,
        tool_call_id,
    } = delta
    {
        dialog.push(DialogMessage::Tool {
            id: tool_call_id.clone(),
            result: content.clone(),
        });
        return;
    }

    let DialogMessage::Assistant {
        content,
        reasoning_content,
        tool_calls,
        ..
    } = last_assistant(dialog)
    else {
        unreachable!()
    };
    match delta {
        AgentMessageDelta::Content(text) => content.push_str(text),
        AgentMessageDelta::ReasoningContent(text) => reasoning_content.push_str(text),
        AgentMessageDelta::ToolCall {
            name,
            arguments,
            tool_call_id,
        } if !name.is_empty() => tool_calls.push(ToolCall {
            id: tool_call_id.clone(),
            name: name.clone(),
            arguments: arguments.clone(),
        }),
        AgentMessageDelta::ToolCall {
            arguments,
            tool_call_id,
            ..
        } => {
            let tool_call = if tool_call_id.is_empty() {
                tool_calls.last_mut()
            } else {
                tool_calls
                    .iter_mut()
                    .find(|tool_call| tool_call.id == *tool_call_id)
            }
            .unwrap();
            tool_call.arguments.push_str(arguments);
        }
        AgentMessageDelta::ToolResult { .. } | AgentMessageDelta::Finish => unreachable!(),
    }
}

pub(crate) fn push_encrypted_reasoning(dialog: &mut Vec<DialogMessage>, item: serde_json::Value) {
    let DialogMessage::Assistant {
        encrypted_reasoning,
        ..
    } = last_assistant(dialog)
    else {
        unreachable!()
    };
    encrypted_reasoning.0.push(item);
}

/// The assistant message at the end of the dialog, started if there is none.
fn last_assistant(dialog: &mut Vec<DialogMessage>) -> &mut DialogMessage {
    if !matches!(dialog.last(), Some(DialogMessage::Assistant { .. })) {
        dialog.push(DialogMessage::Assistant {
            content: String::new(),
            reasoning_content: String::new(),
            tool_calls: Vec::new(),
            encrypted_reasoning: EncryptedReasoning::default(),
        });
    }
    dialog.last_mut().unwrap()
}
