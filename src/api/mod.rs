mod chat_completion;
mod responses;

use std::sync::mpsc;

use crate::{
    agent::{self, ApiDialect, DialogMessage, Thinking},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    tool::Tool,
};

pub(crate) type DeltaResult = Result<AgentMessageDelta, AgentMessageError>;

pub(crate) enum StreamOutcome {
    Finished,
    Error(AgentMessageError),
    ToolCalls,
}

pub(crate) fn stream_response(
    api_dialect: ApiDialect,
    dialog: &mut Vec<DialogMessage>,
    model: &str,
    thinking: &Thinking,
    tools: &[Tool],
    api_base_url: &str,
    api_key: &str,
    tx: &mpsc::Sender<DeltaResult>,
) -> StreamOutcome {
    match api_dialect {
        ApiDialect::DeepSeekChatCompletions => chat_completion::stream_response(
            dialog,
            model,
            thinking,
            tools,
            api_base_url,
            api_key,
            tx,
        ),
        ApiDialect::DeepSeekResponses => {
            responses::stream_response(dialog, model, thinking, tools, api_base_url, api_key, tx)
        }
    }
}

pub(crate) fn emit_delta(
    dialog: &mut Vec<DialogMessage>,
    tx: &mpsc::Sender<DeltaResult>,
    delta: AgentMessageDelta,
) {
    agent::apply_delta(dialog, &delta);
    tx.send(Ok(delta)).unwrap();
}
