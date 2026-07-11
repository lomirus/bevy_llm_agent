pub mod agent;
pub mod prelude;
pub mod tool;

use bevy::tasks::futures_lite::StreamExt;
pub use rig::agent::MultiTurnStreamItem as MultiTurnItem;
pub use rig::message::Message;
pub use rig::message::ToolResultContent;
pub use rig::streaming::StreamedAssistantContent as AssistantContent;
pub use rig::streaming::StreamedUserContent as UserContent;

use agent::{Agent, AgentStatus};
use bevy::prelude::*;
use rig::providers::deepseek::StreamingCompletionResponse as CompletionResponse;
use rig::streaming::StreamingChat;
use tokio::sync::mpsc::unbounded_channel;

#[derive(Message)]
pub struct AgentMessage {
    pub entity: Entity,
    pub delta: MultiTurnItem<CompletionResponse>,
}

#[derive(Message)]
pub struct UserMessage {
    pub entity: Entity,
    pub prompt: Message,
}

impl UserMessage {
    pub fn new(entity: Entity, prompt: impl Into<Message>) -> Self {
        UserMessage {
            entity,
            prompt: prompt.into(),
        }
    }
}

pub struct LlmAgentPlugin;

impl Plugin for LlmAgentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<UserMessage>()
            .add_message::<AgentMessage>()
            .add_systems(FixedUpdate, (read_user_message, write_agent_message));
    }
}

fn read_user_message(mut agents: Query<&mut Agent>, mut reader: MessageReader<UserMessage>) {
    for UserMessage { entity, prompt } in reader.read() {
        let mut agent = agents.get_mut(*entity).unwrap();
        let req = agent.agent.stream_chat(prompt, agent.dialog.clone());
        let (tx, rx) = unbounded_channel();
        agent.status = AgentStatus::Streaming(rx);
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

fn write_agent_message(
    mut agents: Query<(Entity, &mut Agent)>,
    mut writer: MessageWriter<AgentMessage>,
) {
    for (entity, mut agent) in agents.iter_mut() {
        if let AgentStatus::Streaming(receiver) = &mut agent.status {
            while let Ok(delta) = receiver.try_recv() {
                writer.write(AgentMessage { entity, delta });
            }
        }
    }
}
