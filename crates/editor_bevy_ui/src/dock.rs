use bevy::prelude::*;

pub struct DockPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DockSystemSet;

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, DockSystemSet);

        app.init_resource::<DockColors>();

        app.add_systems(Update, (
                panel_node_sync,
                panel_split_sync,
            ).in_set(DockSystemSet),
        );
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

#[derive(PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

fn panel_split_sync(
    mut commands : Commands,
    mut query : Query<(Entity, &mut PanelSplit), Changed<PanelSplit>>,
    colors : Res<DockColors>,
) {
    for (entity, panel_split) in query.iter_mut() {

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
            }
        ));
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