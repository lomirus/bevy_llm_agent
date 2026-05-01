use bevy::prelude::*;
use bevy_llm::{
    Client, LlmPlugin, MultiTurnStreamItem, StreamedAssistantContent, agent::Agent, prelude::*,
};

fn setup(mut commands: Commands) {
    let client = Client::from_env().unwrap();
    let agent = client.agent(DEEPSEEK_V4_FLASH).build();
    let mut agent = Agent::new(agent);
    agent.streaming_chat(
        "Hello, world! Tell me a story.",
        Vec::<bevy_llm::Message>::new(),
    );
    commands.spawn(Camera2d);
    commands.spawn((Text::default(), agent));
}

fn update_text(
    mut texts: Query<&mut Text>,
    mut stream_messages: MessageReader<bevy_llm::StreamMessage>,
) {
    for stream_message in stream_messages.read() {
        let mut text = texts.get_mut(stream_message.entity).unwrap();
        if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(delta)) =
            &stream_message.delta
        {
            text.push_str(delta.text())
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LlmPlugin))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_text)
        .run();
}
