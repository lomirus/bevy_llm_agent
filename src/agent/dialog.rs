use crate::{
    agent::{ApiDialect, DialogMessage, RawOutput, ToolCall},
    messages::agent_message::AgentMessageDelta,
};

pub(crate) fn apply_delta(dialog: &mut Vec<DialogMessage>, delta: &AgentMessageDelta) {
    match delta {
        AgentMessageDelta::Content(text) => last_assistant(dialog).content.push_str(text),
        AgentMessageDelta::ReasoningContent(text) => {
            last_assistant(dialog).reasoning_content.push_str(text);
        }
        AgentMessageDelta::ToolCallStart { id, name } => {
            last_assistant(dialog).tool_calls.push(ToolCall {
                id: id.clone(),
                name: name.clone(),
                arguments: String::new(),
            });
        }
        AgentMessageDelta::ToolCallArguments { id, arguments } => {
            // The API layer only sends arguments for calls it started.
            let tool_calls = last_assistant(dialog).tool_calls;
            if let Some(tool_call) = tool_calls.iter_mut().rev().find(|call| call.id == *id) {
                tool_call.arguments.push_str(arguments);
            }
        }
        AgentMessageDelta::ToolResult { id, content } => dialog.push(DialogMessage::Tool {
            id: id.clone(),
            result: content.clone(),
        }),
        AgentMessageDelta::Finished(_) => {}
    }
}

pub(crate) fn push_raw_output(
    dialog: &mut Vec<DialogMessage>,
    dialect: ApiDialect,
    item: serde_json::Value,
) {
    last_assistant(dialog).raw_output.push(dialect, item);
}

struct AssistantMut<'a> {
    content: &'a mut String,
    reasoning_content: &'a mut String,
    tool_calls: &'a mut Vec<ToolCall>,
    raw_output: &'a mut RawOutput,
}

/// The assistant message at the end of the dialog, started if there is none.
fn last_assistant(dialog: &mut Vec<DialogMessage>) -> AssistantMut<'_> {
    if !matches!(dialog.last(), Some(DialogMessage::Assistant { .. })) {
        dialog.push(DialogMessage::Assistant {
            content: String::new(),
            reasoning_content: String::new(),
            tool_calls: Vec::new(),
            raw_output: RawOutput::default(),
        });
    }
    let Some(DialogMessage::Assistant {
        content,
        reasoning_content,
        tool_calls,
        raw_output,
    }) = dialog.last_mut()
    else {
        unreachable!("an assistant message was just ensured");
    };
    AssistantMut {
        content,
        reasoning_content,
        tool_calls,
        raw_output,
    }
}
