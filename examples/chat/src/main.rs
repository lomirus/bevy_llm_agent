mod auto_scroll;
mod ui;

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    prelude::*,
};
use bevy_llm_agent::{prelude::*, provider::deepseek};

use auto_scroll::AutoScrollToBottomPlugin;
use ui::{chat_ui, enable_send_button, update_chat_messages};

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn(
        Agent::builder(deepseek::FLASH)
            .thinking(deepseek::Thinking::Off)
            .build()
            .unwrap(),
    );
    commands.spawn_scene(chat_ui());
}

fn main() {
    App::new()
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((
            DefaultPlugins,
            FeathersPlugins,
            LlmAgentPlugin,
            AutoScrollToBottomPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_chat_messages)
        .add_observer(enable_send_button)
        .run();
}
