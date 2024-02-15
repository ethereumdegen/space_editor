use bevy::prelude::*;
use space_editor_bevy_ui::dock::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(space_editor_bevy_ui::UiPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(NodeBundle {
        style: Style {
            padding: UiRect::all(Val::Px(7.)),
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            ..default()
        },
        ..default()
    }).with_children(|children| {
        children.spawn(PanelSplit {
            orientation: SplitOrientation::Horizontal,
            ratios: vec![0.2, 0.5, 0.3],
        }).with_children(|children| {
            children.spawn(Panel {
                name: "Test".to_string(),
            });

            children.spawn(Panel {
                name: "Test2".to_string(),
            });

            children.spawn(PanelSplit {
                orientation: SplitOrientation::Vertical,
                ratios: vec![0.5, 0.5],
            }).with_children(|children| {
                children.spawn(Panel {
                    name: "Test3".to_string(),
                });

                children.spawn(Panel {
                    name: "Test4".to_string(),
                });
            });
        });
    });
}