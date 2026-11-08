mod chat_completion;
mod responses;

use std::sync::mpsc;

use crate::{
    agent::{self, Agent, ApiDialect, DialogMessage},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    tool::Tool,
};

pub(crate) type DeltaResult = Result<AgentMessageDelta, AgentMessageError>;

pub(crate) enum StreamOutcome {
    Finished,
    ToolCalls,
}

pub(crate) fn stream_response(
    agent: &mut Agent,
    tools: &[Tool],
    tx: &mpsc::Sender<DeltaResult>,
) -> Result<StreamOutcome, AgentMessageError> {
    let Agent {
        api_dialect,
        dialog,
        model,
        reasoning,
        api_base_url,
        api_key,
    } = agent;
    match api_dialect {
        ApiDialect::DeepSeekChatCompletions | ApiDialect::MoonshotChatCompletions => {
            chat_completion::stream_response(
                dialog,
                model,
                *reasoning,
                tools,
                api_base_url,
                api_key,
                tx,
            )
        }
        ApiDialect::DeepSeekResponses => {
            responses::stream_response(dialog, model, *reasoning, tools, api_base_url, api_key, tx)
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
