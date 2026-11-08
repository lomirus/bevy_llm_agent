mod game;
mod tools;
mod ui;

use bevy::{
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_llm_agent::prelude::*;

use game::{ElapsedTime, GamePhase};
use ui::{DialogueView, SelectedCharacter};

fn main() {
    // See issue: https://github.com/bevyengine/bevy/issues/24094
    const FILTER_ICU_PROVIDER: &str = "icu_provider::error=off";

    App::new()
        .init_resource::<SelectedCharacter>()
        .init_resource::<DialogueView>()
        .init_resource::<GamePhase>()
        .insert_resource(ElapsedTime(0.0))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},{FILTER_ICU_PROVIDER}"),
                ..default()
            }),
            FeathersPlugins,
            MeshPickingPlugin,
            LlmAgentPlugin,
            ui::AutoScrollToBottomPlugin,
        ))
        .add_systems(Startup, (ui::setup, ui::select_strongest_character).chain())
        .add_systems(
            Update,
            (
                ui::update_dialogue_panel,
                ui::update_character_detail,
                ui::update_character_sprites,
                game::exit_on_agent_error,
                game::check_awaiting_moves,
                game::advance_world,
                ui::sync_continue_button,
            ),
        )
        .run();
}
