use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    feathers::{
        palette,
        theme::{ThemeBackgroundColor, ThemeBorderColor},
        tokens,
    },
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    ui::widget::ViewportNode,
};
use bevy_llm_agent::{
    agent::{Agent, DialogMessage, Thinking},
    provider::deepseek,
    tool::AgentTools,
};
use rand::random_range;

use crate::{
    game::{Character, Coordinate, character_system_prompt},
    tools::{DevourCharacter, GetCurrentInfo, MoveTo},
};

use super::{
    character::{
        character_detail_panel, character_list_panel, character_list_row, on_character_map_click,
    },
    dialogue::dialogue_section,
};

/// Map side length in meters (matches [`Coordinate::random`] range of ±10).
const MAP_SIZE_M: f32 = 20.0;
/// Character collision / render radius in meters.
const CHARACTER_RADIUS_M: f32 = 0.5;
/// Render layer for the map camera and world entities (UI stays on layer 0).
const MAP_RENDER_LAYER: usize = 1;

#[derive(Component, Clone, Default)]
pub(crate) struct MapViewport;

#[derive(Component, Clone, Default)]
pub(crate) struct MapCamera;

fn character_agent(id: u8) -> Agent {
    let mut agent = Agent::new(deepseek::V4_FLASH, Thinking::Off);
    agent
        .dialog
        .push(DialogMessage::system(&character_system_prompt(id)));
    agent
}

fn character_scene(character: Character) -> impl Scene {
    let agent = character_agent(character.id);
    let transform = Transform::from_xyz(character.coordinate.x, character.coordinate.y, 0.0);
    let render_layers = RenderLayers::layer(MAP_RENDER_LAYER);
    bsn! {
        template_value(character)
        template_value(agent)
        AgentTools [
            @GetCurrentInfo,
            @MoveTo,
            @DevourCharacter,
        ]
        Mesh2d(asset_value(Circle::new(CHARACTER_RADIUS_M)))
        MeshMaterial2d::<ColorMaterial>(asset_value(palette::LIGHT_GRAY_1))
        template_value(transform)
        template_value(render_layers)
        on(on_character_map_click)
    }
}

fn map_panel() -> impl Scene {
    bsn! {
        Node {
            height: percent(100),
            aspect_ratio: {Some(1.0)},
            flex_shrink: 0.0,
            border: UiRect::all(px(1)),
            padding: px(8),
        }
        ThemeBackgroundColor(tokens::PANE_BODY_BG)
        ThemeBorderColor(tokens::PANE_HEADER_BORDER)
        Children [(
            Node {
                width: percent(100),
                height: percent(100),
            }
            MapViewport
        )]
    }
}

pub(crate) fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // These camera components are not bsn-friendly (missing Clone/Default/FromTemplate).
    commands.spawn((Camera2d, IsDefaultUiCamera));

    let mut map_image = Image::new_uninit(
        default(),
        TextureDimension::D2,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    );
    map_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let map_image = images.add(map_image);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(palette::GRAY_1),
            ..default()
        },
        RenderTarget::Image(map_image.into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: MAP_SIZE_M,
                height: MAP_SIZE_M,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(MAP_RENDER_LAYER),
        MapCamera,
    ));

    let mut characters = Vec::new();
    for id in 1..=10 {
        let character = Character {
            id,
            combat_power: random_range::<u32, _>(1..=1000),
            moving_speed: random_range::<f32, _>(0.5..2.0),
            coordinate: Coordinate::random(),
            moving_target: None,
            is_dead: false,
        };
        let entity = commands
            .spawn_scene(character_scene(character.clone()))
            .id();
        characters.push((entity, character));
    }

    characters.sort_by_key(|(_, character)| character.id);

    commands.spawn_scene(game_ui(&characters));
}

pub(crate) fn attach_map_viewport(
    mut commands: Commands,
    map_viewport: Single<Entity, With<MapViewport>>,
    map_camera: Single<Entity, With<MapCamera>>,
) {
    commands
        .entity(*map_viewport)
        .insert(ViewportNode::new(*map_camera));
}

fn game_ui(characters: &[(Entity, Character)]) -> impl Scene {
    let rows: Vec<_> = characters
        .iter()
        .map(|(entity, character)| character_list_row(*entity, character))
        .collect();

    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            padding: px(16),
            column_gap: px(16),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            // Character inspector
            (
                Node {
                    min_width: px(240),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                }
                Children [
                    ({character_detail_panel()}),
                    ({character_list_panel(rows)}),
                ]
            ),
            // 20m x 20m square game map
            ({map_panel()}),
            // Right-hand dialogue panel
            ({dialogue_section()}),
        ]
    }
}
