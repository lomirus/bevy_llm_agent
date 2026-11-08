mod chat_completion;
mod responses;

use std::io::{BufRead, BufReader};

use serde::Serialize;

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
    /// A reasoning output item to send back unchanged, see [`EncryptedReasoning`](crate::agent::EncryptedReasoning).
    EncryptedReasoning(serde_json::Value),
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
    match agent.api_dialect {
        ApiDialect::ChatCompletions => chat_completion::stream_response(agent, tools, tx),
        ApiDialect::Responses => responses::stream_response(agent, tools, tx),
    }
}

pub(crate) fn emit(tx: &EventSender, delta: AgentMessageDelta) -> Result<(), AgentMessageError> {
    send(tx, StreamEvent::Delta(delta))
}

/// Fails once the agent stops listening, e.g. after it is despawned, which ends the request.
pub(crate) fn send(tx: &EventSender, event: StreamEvent) -> Result<(), AgentMessageError> {
    tx.send(event)
        .map_err(|_| AgentMessageError::Other("the agent is no longer listening".to_owned()))
}

/// POST `body` to the agent's API and iterate over the `data` of the server-sent events.
fn post_sse(
    agent: &Agent,
    path: &str,
    body: &impl Serialize,
) -> Result<impl Iterator<Item = Result<String, AgentMessageError>>, AgentMessageError> {
    let url = format!("{}{path}", agent.api_base_url);
    let mut response = ureq::post(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", format!("Bearer {}", agent.api_key))
        .send_json(body)
        .map_err(other)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(AgentMessageError::Other(if body.is_empty() {
            format!("http status: {status}")
        } else {
            body
        }));
    }

    let lines = BufReader::new(response.into_body().into_reader()).lines();
    Ok(lines.filter_map(|line| match line {
        Ok(line) => line.strip_prefix("data: ").map(|data| Ok(data.to_owned())),
        Err(error) => Some(Err(other(error))),
    }))
}

fn other(error: impl ToString) -> AgentMessageError {
    AgentMessageError::Other(error.to_string())
}
