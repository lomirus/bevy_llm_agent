mod tools;

use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    feathers::{
        FeathersPlugins,
        containers::{subpane, subpane_body, subpane_header},
        controls::{ButtonVariant, FeathersButton, FeathersListRow, FeathersListView},
        dark_theme::create_dark_theme,
        palette,
        theme::{ThemeBackgroundColor, ThemeBorderColor, ThemedText, UiTheme},
        tokens,
    },
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
    render::render_resource::{TextureDimension, TextureFormat, TextureUsages},
    ui::{Selected, widget::ViewportNode},
    ui_widgets::ValueChange,
};
use bevy_llm_agent::{
    AppExt, DEEPSEEK_V4_FLASH, LlmAgentPlugin,
    agent::{Agent, Thinking},
    tool::AgentTools,
};
use rand::random_range;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tools::{DevourCharacter, GetCurrentInfo, MoveTo};

const UI_FONT_SIZE: f32 = 12.0;
/// Map side length in meters (matches `Coordinate::random` range of ±10).
const MAP_SIZE_M: f32 = 20.0;
/// Character collision / render radius in meters.
const CHARACTER_RADIUS_M: f32 = 0.5;
/// Render layer for the map camera and world entities (UI stays on layer 0).
const MAP_RENDER_LAYER: usize = 1;

#[derive(Component, JsonSchema, Serialize, Clone, Default)]
struct Character {
    /// Character ID
    id: u8,
    /// Combat power.
    combat_power: u32,
    /// Moving speed.
    ///
    /// Unit: m/s.
    moving_speed: f32,
    /// Current coordinate of the character
    coordinate: Coordinate,
    /// Coordinate of the moving target
    moving_target: Option<Coordinate>,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default)]
struct Coordinate {
    /// Unit: m
    x: f32,
    /// Unit: m
    y: f32,
}

impl Coordinate {
    fn random() -> Self {
        Self {
            x: random_range(-10.0..10.0),
            y: random_range(-10.0..10.0),
        }
    }

    fn dist(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Resource)]
struct ElapsedTime(f32);

#[derive(Resource, Default)]
struct SelectedCharacter(Option<Entity>);

#[derive(Component, Clone)]
struct CharacterListItem(Entity);

impl Default for CharacterListItem {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Clone, Default)]
struct CharacterDetailBody;

/// Marks the UI node that displays the map camera render target.
#[derive(Component, Clone, Default)]
struct MapViewport;

#[derive(Component, Clone, Default)]
struct MapCamera;

fn character_scene(character: Character) -> impl Scene {
    let transform = Transform::from_xyz(character.coordinate.x, character.coordinate.y, 0.0);
    let render_layers = RenderLayers::layer(MAP_RENDER_LAYER);
    bsn! {
        template_value(character)
        Agent::new(DEEPSEEK_V4_FLASH, Thinking::Off)
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

fn character_detail_values(character: &Character) -> [String; 5] {
    [
        character.id.to_string(),
        character.combat_power.to_string(),
        format!("{:.2} m/s", character.moving_speed),
        format!(
            "({:.1}, {:.1})",
            character.coordinate.x, character.coordinate.y
        ),
        match &character.moving_target {
            Some(t) => format!("({:.1}, {:.1})", t.x, t.y),
            None => "None".into(),
        },
    ]
}

fn detail_row(label: &str, value: String) -> impl Scene {
    let label = label.to_string();
    bsn! {
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
        }
        Children [
            (
                Text({label})
                ThemedText
                TextLayout::new(Justify::Left, LineBreak::NoWrap)
                TextFont {
                    font_size: FontSize::Px(UI_FONT_SIZE),
                }
            ),
            (
                Text({value})
                ThemedText
                TextLayout::new(Justify::Right, LineBreak::NoWrap)
                TextFont {
                    font_size: FontSize::Px(UI_FONT_SIZE),
                }
            ),
        ]
    }
}

fn character_detail_panel() -> impl Scene {
    bsn! {
        subpane()
        Children [
            subpane_header() Children [
                (Text("Selected Character") ThemedText)
            ],
            (
                subpane_body()
                CharacterDetailBody
                Children [
                    ({detail_row("ID", String::new())}),
                    ({detail_row("Combat Power", String::new())}),
                    ({detail_row("Moving Speed", String::new())}),
                    ({detail_row("Coordinate", String::new())}),
                    ({detail_row("Moving Target", String::new())}),
                ]
            )
        ]
    }
}

fn character_list_row(entity: Entity, character: &Character) -> impl Scene {
    let label = format!("ID {}: {}", character.id, character.combat_power);
    bsn! {
        @FeathersListRow
        CharacterListItem(entity)
        Children [(
            Text({label})
            ThemedText
        )]
    }
}

fn character_list_panel(rows: impl SceneList) -> impl Scene {
    bsn! {
        subpane()
        Node {
            flex_grow: 1.0,
            min_height: px(0),
        }
        Children [
            subpane_header() Children [
                (Text("Characters") ThemedText)
            ],
            (
                subpane_body()
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                }
                Children [(
                    @FeathersListView {
                        @rows: {Box::new(rows) as Box<dyn SceneList>}
                    }
                    Node {
                        flex_grow: 1.0,
                        min_height: px(0),
                    }
                    on(on_character_list_select)
                )]
            )
        ]
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
        };
        let entity = commands
            .spawn_scene(character_scene(character.clone()))
            .id();
        characters.push((entity, character));
    }

    characters.sort_by_key(|(_, character)| character.id);

    commands.spawn_scene(game_ui(&characters));
}

fn attach_map_viewport(
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
            (
                Node {
                    flex_grow: 1.0,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(12),
                }
                Children [
                    // Current dialogue content
                    (
                        Node {
                            flex_grow: 1.0,
                            border: UiRect::all(px(1)),
                            padding: px(12),
                        }
                        ThemeBackgroundColor(tokens::PANE_BODY_BG)
                        ThemeBorderColor(tokens::PANE_HEADER_BORDER)
                        Children [(
                            Text("Dialogue")
                            ThemedText
                        )]
                    ),
                    // Dialogue navigation controls
                    (
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                        }
                        Children [
                            // Previous dialogue message
                            (
                                @FeathersButton {
                                    @caption: bsn! {
                                        Text("Previous")
                                        ThemedText
                                    }
                                }
                            ),
                            // Next dialogue message
                            (
                                @FeathersButton {
                                    @caption: bsn! {
                                        Text("Next")
                                        ThemedText
                                    }
                                }
                            ),
                            // Continue dialogue
                            (
                                @FeathersButton {
                                    @caption: bsn! {
                                        Text("Continue Dialogue")
                                        ThemedText
                                    }
                                    @variant: ButtonVariant::Primary,
                                }
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn apply_character_selection(
    character: Entity,
    selected: &mut SelectedCharacter,
    items: &Query<(Entity, &CharacterListItem, Has<Selected>)>,
    commands: &mut Commands,
) {
    selected.0 = Some(character);
    for (row, CharacterListItem(item), is_selected) in items {
        if *item == character {
            if !is_selected {
                commands.entity(row).insert(Selected);
            }
        } else if is_selected {
            commands.entity(row).remove::<Selected>();
        }
    }
}

fn on_character_list_select(
    change: On<ValueChange<Entity>>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut selected: ResMut<SelectedCharacter>,
    mut commands: Commands,
) {
    let Ok((_, CharacterListItem(character), _)) = items.get(change.event().value) else {
        return;
    };
    apply_character_selection(*character, &mut selected, &items, &mut commands);
}

fn on_character_map_click(
    click: On<Pointer<Click>>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut selected: ResMut<SelectedCharacter>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    apply_character_selection(click.entity, &mut selected, &items, &mut commands);
}

fn select_strongest_character(
    mut selected: ResMut<SelectedCharacter>,
    characters: Query<(Entity, &Character)>,
    items: Query<(Entity, &CharacterListItem, Has<Selected>)>,
    mut commands: Commands,
) {
    let Some(character) = characters
        .iter()
        .max_by_key(|(_, character)| character.combat_power)
        .map(|(entity, _)| entity)
    else {
        return;
    };
    apply_character_selection(character, &mut selected, &items, &mut commands);
}

fn update_character_detail(
    selected: Res<SelectedCharacter>,
    characters: Query<&Character>,
    body: Single<&Children, With<CharacterDetailBody>>,
    rows: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    if !selected.is_changed() {
        return;
    }
    let values = selected
        .0
        .and_then(|entity| characters.get(entity).ok())
        .map(character_detail_values)
        .unwrap_or_default();
    for (row, value) in body.iter().zip(values) {
        let Some(&value_entity) = rows.get(row).ok().and_then(|c| c.get(1)) else {
            continue;
        };
        if let Ok(mut text) = texts.get_mut(value_entity) {
            text.0 = value;
        }
    }
}

fn update_character_sprites(
    selected: Res<SelectedCharacter>,
    mut characters: Query<(
        Entity,
        Ref<Character>,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let selection_changed = selected.is_changed();
    for (entity, character, mut transform, material) in &mut characters {
        if character.is_changed() {
            transform.translation.x = character.coordinate.x;
            transform.translation.y = character.coordinate.y;
        }
        if selection_changed {
            if let Some(mut material) = materials.get_mut(&material.0) {
                material.color = if selected.0 == Some(entity) {
                    palette::ACCENT
                } else {
                    palette::LIGHT_GRAY_1
                };
            }
        }
    }
}

fn main() {
    App::new()
        .init_resource::<SelectedCharacter>()
        .insert_resource(ElapsedTime(0.0))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: DEFAULT_FILTER.to_owned(),
                ..default()
            }),
            FeathersPlugins,
            MeshPickingPlugin,
            LlmAgentPlugin,
        ))
        .add_agent_tool::<GetCurrentInfo>()
        .add_agent_tool::<MoveTo>()
        .add_agent_tool::<DevourCharacter>()
        .add_systems(
            Startup,
            (
                setup,
                ApplyDeferred,
                attach_map_viewport,
                select_strongest_character,
            )
                .chain(),
        )
        .add_systems(Update, (update_character_detail, update_character_sprites))
        .run();
}
