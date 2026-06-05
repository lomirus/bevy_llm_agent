use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use rig::{providers::deepseek::CompletionModel, streaming::StreamingChat};
use tokio::sync::mpsc::unbounded_channel;

use crate::agent::agent_status::AgentStatus;

#[derive(Component)]
pub struct Agent {
    pub(crate) agent: rig::agent::Agent<CompletionModel>,
    pub dialog: Vec<rig::message::Message>,
    pub(crate) status: AgentStatus,
}

impl Agent {
    pub fn streaming_chat(
        &mut self,
        prompt: impl Into<rig::message::Message> + rig::wasm_compat::WasmCompatSend,
    ) {
        let req = self.agent.stream_chat(prompt, self.dialog.clone());

        let (tx, rx) = unbounded_channel();
        self.status = AgentStatus::Streaming(rx);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let mut stream = req.await;
                while let Some(Ok(delta)) = stream.next().await {
                    tx.send(delta).unwrap();
                }
            });
        });
    }
}
