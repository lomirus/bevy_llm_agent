mod chat_completion;
mod interactions;
mod messages;
mod responses;

use std::io::{BufRead, BufReader, Lines};

use serde::Serialize;
use ureq::BodyReader;

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
    /// An output item to send back unchanged, see [`RawOutput`](crate::agent::RawOutput).
    RawOutput(ApiDialect, serde_json::Value),
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
        ApiDialect::Messages => messages::stream_response(agent, tools, tx),
        ApiDialect::Interactions => interactions::stream_response(agent, tools, tx),
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

/// POST `body` to the agent's API with the API key in a bearer `Authorization` header and
/// iterate over the `data` of the server-sent events.
fn post_sse(
    agent: &Agent,
    path: &str,
    body: &impl Serialize,
) -> Result<SseData, AgentMessageError> {
    let authorization = format!("Bearer {}", agent.api_key);
    post_sse_with_headers(agent, path, &[("Authorization", &authorization)], body)
}

/// POST `body` to the agent's API with `headers`, which carry the credentials, and iterate over
/// the `data` of the server-sent events.
fn post_sse_with_headers(
    agent: &Agent,
    path: &str,
    headers: &[(&str, &str)],
    body: &impl Serialize,
) -> Result<SseData, AgentMessageError> {
    let url = format!("{}{path}", agent.api_base_url);
    let mut request = ureq::post(&url)
        .config()
        .http_status_as_error(false)
        .build();
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let mut response = request.send_json(body).map_err(other)?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        return Err(AgentMessageError::Other(if body.is_empty() {
            format!("http status: {status}")
        } else {
            body
        }));
    }

    Ok(SseData(
        BufReader::new(response.into_body().into_reader()).lines(),
    ))
}

/// The `data` of the server-sent events in a response body.
struct SseData(Lines<BufReader<BodyReader<'static>>>);

impl Iterator for SseData {
    type Item = Result<String, AgentMessageError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next()? {
                Ok(line) => {
                    if let Some(data) = line.strip_prefix("data: ") {
                        return Some(Ok(data.to_owned()));
                    }
                }
                Err(error) => return Some(Err(other(error))),
            }
        }
    }
}

fn other(error: impl ToString) -> AgentMessageError {
    AgentMessageError::Other(error.to_string())
}
