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

    commands.spawn(Panel {
        name: "Test".to_string(),
    });
}