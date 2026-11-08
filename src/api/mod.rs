mod chat_completion;
mod responses;

use crate::{
    agent::{Agent, ApiDialect},
    messages::agent_message::{AgentMessageDelta, AgentMessageError},
    tool::Tool,
};

pub(crate) enum StreamOutcome {
    Finished,
    ToolCalls,
}

/// Sent from a request worker to the agent.
pub(crate) enum StreamEvent {
    Delta(AgentMessageDelta),
    /// The last event of a request.
    End(Result<StreamOutcome, AgentMessageError>),
}

pub(crate) type EventSender = crossbeam_channel::Sender<StreamEvent>;

/// Send one request for the agent's dialog, streaming deltas through `tx`.
pub(crate) fn stream_response(
    agent: &Agent,
    tools: &[Tool],
    tx: &EventSender,
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

/// Fails once the agent stops listening, e.g. after it is despawned, which ends the request.
pub(crate) fn emit(tx: &EventSender, delta: AgentMessageDelta) -> Result<(), AgentMessageError> {
    tx.send(StreamEvent::Delta(delta))
        .map_err(|_| AgentMessageError::Other("the agent is no longer listening".to_owned()))
}
