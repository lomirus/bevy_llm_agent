mod auto_scroll;
mod character;
mod dialogue;
mod scene;

const UI_FONT_SIZE: f32 = 12.0;

pub(crate) use auto_scroll::AutoScrollToBottomPlugin;
pub(crate) use character::{
    SelectedCharacter, select_strongest_character, update_character_detail,
    update_character_sprites,
};
pub(crate) use dialogue::{DialogueView, sync_continue_button, update_dialogue_panel};
pub(crate) use scene::setup;
