use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{FeathersListRow, FeathersListView},
        palette,
        theme::ThemedText,
    },
    prelude::*,
    ui::Selected,
    ui_widgets::ValueChange,
};

use crate::game::Character;

use super::UI_FONT_SIZE;

#[derive(Resource, Default)]
pub(crate) struct SelectedCharacter(pub(super) Option<Entity>);

#[derive(Component, Clone)]
pub(crate) struct CharacterListItem(Entity);

impl Default for CharacterListItem {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Clone, Default)]
pub(crate) struct CharacterDetailBody;

fn character_detail_values(character: &Character) -> [String; 6] {
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
        if character.is_dead {
            "Yes".into()
        } else {
            "No".into()
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

pub(super) fn character_detail_panel() -> impl Scene {
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
                    ({detail_row("Dead", String::new())}),
                ]
            )
        ]
    }
}

pub(super) fn character_list_row(entity: Entity, character: &Character) -> impl Scene {
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

pub(super) fn character_list_panel(rows: impl SceneList) -> impl Scene {
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

pub(super) fn on_character_map_click(
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

pub(crate) fn select_strongest_character(
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

pub(crate) fn update_character_detail(
    selected: Res<SelectedCharacter>,
    characters: Query<Ref<Character>>,
    body: Single<&Children, With<CharacterDetailBody>>,
    rows: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    let Some(entity) = selected.0 else {
        return;
    };
    let Ok(character) = characters.get(entity) else {
        return;
    };
    if !selected.is_changed() && !character.is_changed() {
        return;
    }
    let values = character_detail_values(&character);
    for (row, value) in body.iter().zip(values) {
        let Some(&value_entity) = rows.get(row).ok().and_then(|c| c.get(1)) else {
            continue;
        };
        if let Ok(mut text) = texts.get_mut(value_entity) {
            text.0 = value;
        }
    }
}

pub(crate) fn update_character_sprites(
    selected: Res<SelectedCharacter>,
    mut characters: Query<(
        Entity,
        Ref<Character>,
        &mut Transform,
        &mut Visibility,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let selection_changed = selected.is_changed();
    for (entity, character, mut transform, mut visibility, material) in &mut characters {
        if character.is_changed() {
            transform.translation.x = character.coordinate.x;
            transform.translation.y = character.coordinate.y;
            *visibility = if character.is_dead {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
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
