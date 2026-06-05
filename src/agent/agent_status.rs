use rig::{agent::MultiTurnStreamItem, providers::deepseek::StreamingCompletionResponse};
use tokio::sync::mpsc::UnboundedReceiver;

pub(crate) enum AgentStatus {
    Idle,
    Streaming(UnboundedReceiver<MultiTurnStreamItem<StreamingCompletionResponse>>),
}
