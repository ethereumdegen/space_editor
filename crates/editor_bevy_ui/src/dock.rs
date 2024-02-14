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
            ).in_set(DockSystemSet),
        );
    }
}

#[derive(Resource)]
pub struct DockColors {
    pub panel_background: Color,
    pub dock_background: Color,
}

impl Default for DockColors {
    fn default() -> Self {
        Self {
            panel_background: Color::rgb_u8(32, 31, 35),
            dock_background: Color::DARK_GRAY,
        }
    }
}

#[derive(Component)]
pub struct Panel {
    pub name: String,
}

#[derive(Component)]
pub struct PanelSplit {
    pub orientation: Orientation,
    pub ratios: Vec<f32>,
}

enum Orientation {
    Horizontal,
    Vertical,
}


fn panel_node_sync(
    mut commands : Commands,
    mut query : Query<(Entity, &mut Panel), Changed<Panel>>,
    colors : Res<DockColors>,
) {
    for (entity, mut panel) in query.iter_mut() {
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
                    margin: UiRect {
                        left: Val::Px(5.0),
                        right: Val::Px(5.0),
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                    },
                    ..default()
                },
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