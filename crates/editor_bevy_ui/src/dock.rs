use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

pub struct DockPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DockSystemSet;

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DockSystemSet);

        app.init_resource::<DockColors>();

        app.add_systems(PreUpdate,
        (
            reset_cursor,
        ).in_set(DockSystemSet),);

        app.add_systems(Update, (
                panel_node_sync,
                panel_split_sync,
                fake_splitter_sync,
                hover_fake_splitter,
                update_selected_fake_splitter
            ).in_set(DockSystemSet),
        );

        app.init_resource::<SelectedFakeSplitter>();
    }
}

#[derive(Resource)]
pub struct DockColors {
    pub panel_background: Color,
    pub dock_background: Color,
    pub panel_border_color: Color,
}

impl Default for DockColors {
    fn default() -> Self {
        Self {
            panel_background: Color::rgb_u8(32, 31, 35),
            dock_background: Color::rgb_u8(51, 49, 55),
            panel_border_color: Color::rgb_u8(35, 33, 38),
        }
    }
}

#[derive(Component)]
pub struct Panel {
    pub name: String,
}

#[derive(Component)]
pub struct PanelSplit {
    pub orientation: SplitOrientation,
    pub ratios: Vec<f32>,
}

#[derive(Component, Default, Clone)]
pub struct PanelSplitInternal {
    pub splitters : Vec<Entity>
}

#[derive(PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

fn panel_split_sync(
    mut commands : Commands,
    mut query : Query<(Entity, &mut PanelSplit, Option<&mut PanelSplitInternal>), Changed<PanelSplit>>,
    colors : Res<DockColors>,
) {
    for (entity, panel_split, internal) in query.iter_mut() {

        let mut style = Style {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            display: Display::Grid,
            ..default()
        };

        if panel_split.orientation == SplitOrientation::Horizontal {
            style.grid_template_columns = panel_split.ratios.iter().map(|ratio| GridTrack::flex(*ratio)).collect();
            style.grid_template_rows = vec![GridTrack::flex(1.0)];
            style.column_gap = Val::Px(7.0);
        } else {
            style.grid_template_rows = panel_split.ratios.iter().map(|ratio| GridTrack::flex(*ratio)).collect();
            style.grid_template_columns = vec![GridTrack::flex(1.0)];
            style.row_gap = Val::Px(7.0);
        }

        commands.entity(entity).insert((
            NodeBundle {
                style,
                ..default()
            },
        ));

        let mut internal = if let Some(internal) = internal {
            internal.clone()
        } else {
            PanelSplitInternal { ..default() }
        };
        if panel_split.ratios.len() > 1 {
            while internal.splitters.len() < panel_split.ratios.len() - 1 {
                let id = commands.spawn((
                    NodeBundle {
                        ..default()
                    },
                    FakeSplitter {
                        parent: entity,
                    }
                )).id();
                internal.splitters.push(id);
            }

            while internal.splitters.len() > panel_split.ratios.len() - 1 {
                commands.entity(internal.splitters.pop().unwrap()).despawn_recursive();
            }
        } else {
            for e in internal.splitters.iter() {
                commands.entity(*e).despawn_recursive();
            }
            internal.splitters.clear();
        }

        commands.entity(entity).insert(internal);
    }
}

fn fake_splitter_sync(
    mut commands : Commands,
    parents : Query<(&Node, &GlobalTransform, &PanelSplit, &PanelSplitInternal, &Children)>,
    splitter_childs: Query<(&Node, &GlobalTransform), Without<FakeSplitter>>,
    mut splitters : Query<(&mut Style, &FakeSplitter)>,

) {
    for (node, transform, split, split_internal, children) in parents.iter() {
        let split_rect = node.logical_rect(transform);
        let splitter_count = split.ratios.len();
        for (idx, splitter_entity) in split_internal.splitters.iter().enumerate() {
            if let Ok((mut style, splitter)) = splitters.get_mut(*splitter_entity) {
                let child_id = *children.get(idx).unwrap();
                let next_child_id = *children.get(idx + 1).unwrap();

                let child_node = splitter_childs.get(child_id).unwrap();
                let next_child_node = splitter_childs.get(next_child_id).unwrap();

                let child_rect = child_node.0.logical_rect(child_node.1);

                style.position_type = PositionType::Absolute;
                if split.orientation == SplitOrientation::Horizontal {
                    style.left = Val::Px(child_rect.max.x);
                    style.top = Val::Px(split_rect.min.y);
                    style.height = Val::Px(split_rect.height());
                    style.width = Val::Px(7.0 - 2.0);
                } else {
                    style.top = Val::Px(child_rect.max.y);
                    style.left = Val::Px(split_rect.min.x);
                    style.width = Val::Px(split_rect.width());
                    style.height = Val::Px(7.0 - 2.0);
                }

                // commands.entity(*splitter_entity).insert(BackgroundColor(Color::RED));
            }
        }
    }
}

fn reset_cursor(
    mut windows : Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        error!("Failed to get primary window");
        return;
    };

    window.cursor.icon = CursorIcon::Default;
}

#[derive(Resource, DerefMut, Deref, Default)]
pub struct SelectedFakeSplitter(Option<Entity>);

fn update_selected_fake_splitter(
    mut query : Query<(Entity, &Node, &GlobalTransform, &FakeSplitter)>,
    mut parents : Query<(&Node, &GlobalTransform, &mut PanelSplit, &PanelSplitInternal)>,
    mut windows : Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_motion : EventReader<MouseMotion>,
    mut mouse_buttons : Res<Input<MouseButton>>,
    mut selected : ResMut<SelectedFakeSplitter>,
    mut ui_scale : ResMut<UiScale>
) {
    let Some(selected_entity) = selected.0.clone() else {
        return;
    };

    let Ok(mut window) = windows.get_single_mut() else {
        error!("Failed to get primary window");
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let mut mouse_move = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Left) {
        for event in mouse_motion.read() {
            mouse_move += event.delta;
        }
    }
    mouse_motion.clear();

    if mouse_buttons.just_released(MouseButton::Left) || !mouse_buttons.pressed(MouseButton::Left) {
        selected.0 = None;
        return;
    }

    let Ok((entity, node, transform, splitter)) = query.get_mut(selected_entity) else {
        selected.0 = None;
        return;
    };

    let split_rect = node.logical_rect(transform);

    let Ok((parent_node, parent_transform, mut panel_split, internal)) = parents.get_mut(splitter.parent) else {
        selected.0 = None;
        return;
    };

    let parent_rect = parent_node.logical_rect(parent_transform);
    let parent_phys_rect = parent_node.physical_rect(parent_transform, window.scale_factor(), ui_scale.0);

    let ratio_idx = internal.splitters.iter().position(|r| *r == entity).unwrap();
    let ratio_sum = panel_split.ratios.iter().sum::<f32>();

    match panel_split.orientation {
        SplitOrientation::Horizontal => {
            let delta = (mouse_move.x / parent_phys_rect.width()) * ratio_sum;

            let two_sum = panel_split.ratios[ratio_idx] + panel_split.ratios[ratio_idx + 1];

            panel_split.ratios[ratio_idx] += delta;
            panel_split.ratios[ratio_idx + 1] -= delta;
            
            panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].max(0.1);
            panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].max(0.1);

            if delta < 0.0 {
                panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].min(two_sum - panel_split.ratios[ratio_idx]);
                panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].min(two_sum - panel_split.ratios[ratio_idx + 1]);
            } else {
                panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].min(two_sum - panel_split.ratios[ratio_idx + 1]);
                panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].min(two_sum - panel_split.ratios[ratio_idx]);
            }


            info!("Motion: {}", delta);
            info!("Ratio: {}", panel_split.ratios[ratio_idx]);
            info!("Sum: {}", two_sum);
        }
        SplitOrientation::Vertical => {
            let delta = (mouse_move.y / parent_rect.height()) * ratio_sum;
           
            let two_sum = panel_split.ratios[ratio_idx] + panel_split.ratios[ratio_idx + 1];

            panel_split.ratios[ratio_idx] += delta;
            panel_split.ratios[ratio_idx + 1] -= delta;
            
            panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].max(0.1);
            panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].max(0.1);

            if delta < 0.0 {
                panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].min(two_sum - panel_split.ratios[ratio_idx]);
                panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].min(two_sum - panel_split.ratios[ratio_idx + 1]);
            } else {
                panel_split.ratios[ratio_idx] = panel_split.ratios[ratio_idx].min(two_sum - panel_split.ratios[ratio_idx + 1]);
                panel_split.ratios[ratio_idx + 1] = panel_split.ratios[ratio_idx + 1].min(two_sum - panel_split.ratios[ratio_idx]);
            }

            info!("Motion: {}", delta);
            info!("Ratio: {}", panel_split.ratios[ratio_idx]);
        }
    }
}

fn hover_fake_splitter(
    mut query : Query<(Entity, &Node, &GlobalTransform, &FakeSplitter)>,
    mut parents : Query<(&Node, &GlobalTransform, &mut PanelSplit, &PanelSplitInternal)>,
    mut windows : Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_buttons : Res<Input<MouseButton>>,
    mut selected : ResMut<SelectedFakeSplitter>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        error!("Failed to get primary window");
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    for (entity, node, transform, splitter) in query.iter_mut() {
        let split_rect = node.logical_rect(transform);

        if split_rect.contains(cursor_position) {
            if let Ok((node, transform, mut panel_split, internal)) = parents.get_mut(splitter.parent) {
                let parent_rect = node.logical_rect(transform);
                let ratio_idx = internal.splitters.iter().position(|r| *r == entity).unwrap();

                if mouse_buttons.just_pressed(MouseButton::Left) {
                    selected.0 = Some(entity);
                }
                match panel_split.orientation {
                    SplitOrientation::Horizontal => {
                        window.cursor.icon = CursorIcon::ColResize;

                    }
                    SplitOrientation::Vertical => {
                        window.cursor.icon = CursorIcon::RowResize;
                    }
                }
            }
        }
    }
}

fn panel_node_sync(
    mut commands : Commands,
    mut query : Query<(Entity, &mut Panel), Changed<Panel>>,
    colors : Res<DockColors>,
) {
    for (entity, panel) in query.iter_mut() {
        info!("Update panel node: {}", panel.name);
        commands.entity(entity).insert((
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    display: Display::Flex,
                    overflow: Overflow::clip(),
                    
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                border_color: BorderColor(colors.panel_border_color),
                background_color: BackgroundColor(colors.panel_background),
                ..default()
            },
        )).with_children(|children| {
            children.spawn((
                TextBundle::from_section(&panel.name, TextStyle {
                    ..default()
                })
            ));
        });
    }
}

#[derive(Component)]
pub struct FakeSplitter {
    pub parent: Entity,
}