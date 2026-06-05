use bevy::{prelude::*, tasks::futures_lite::StreamExt};
use rig::{
    agent::{MultiTurnStreamItem, NoToolConfig, WithBuilderTools},
    client::{CompletionClient, ProviderClient},
    providers::deepseek::{Client, CompletionModel, StreamingCompletionResponse},
    streaming::StreamingChat,
};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::tool::{Tool, ToolAdapter};

pub struct AgentBuilder<T = NoToolConfig>(rig::agent::AgentBuilder<CompletionModel, (), T>);

impl AgentBuilder<NoToolConfig> {
    pub fn new(model: &str) -> AgentBuilder<NoToolConfig> {
        let client = Client::from_env().unwrap();
        let agent_builder = client.agent(model).default_max_turns(usize::MAX - 1);
        AgentBuilder(agent_builder)
    }

    pub fn tool<T: Tool>(self) -> AgentBuilder<WithBuilderTools> {
        AgentBuilder(self.0.tool(ToolAdapter::<T>::new()))
    }

    pub fn build(self) -> Agent {
        Agent {
            agent: self.0.build(),
            dialog: Vec::new(),
            status: AgentStatus::Idle,
        }
    }
}

impl AgentBuilder<WithBuilderTools> {
    pub fn tool<T: Tool>(self) -> AgentBuilder<WithBuilderTools> {
        AgentBuilder(self.0.tool(ToolAdapter::<T>::new()))
    }

    pub fn build(self) -> Agent {
        Agent {
            agent: self.0.build(),
            dialog: Vec::new(),
            status: AgentStatus::Idle,
        }
    }
}

#[derive(Component)]
pub struct Agent {
    agent: rig::agent::Agent<CompletionModel>,
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

pub(crate) enum AgentStatus {
    Idle,
    Streaming(UnboundedReceiver<MultiTurnStreamItem<StreamingCompletionResponse>>),
}
