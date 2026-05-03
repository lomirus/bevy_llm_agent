use bevy::{prelude::*, ui::UiSystems};

#[derive(Component, FromTemplate)]
pub struct AutoScrollToBottom {
    last_max_y: f32,
}

fn auto_scroll_to_bottom(
    mut query: Query<(&ComputedNode, &mut ScrollPosition, &mut AutoScrollToBottom)>,
) {
    for (node, mut scroll_position, mut auto_scroll) in &mut query {
        let max_y = (node.content_size().y - node.size().y).max(0.0) * node.inverse_scale_factor();
        if max_y != auto_scroll.last_max_y {
            auto_scroll.last_max_y = max_y;
            scroll_position.y = max_y;
        }
    }
}

pub struct AutoScrollToBottomPlugin;

impl Plugin for AutoScrollToBottomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            auto_scroll_to_bottom.in_set(UiSystems::PostLayout),
        );
    }
}
