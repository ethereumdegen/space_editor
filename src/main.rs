use bevy::{prelude::*, window::WindowResolution};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            fit_canvas_to_parent: true,
            title: "Space Editor".into(),
            resolution: WindowResolution::new(1920., 1080.),
            visible: true,
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "editor")]
    {
        use space_editor::SpaceEditorPlugin;

        app.add_plugins(SpaceEditorPlugin);
    }
    app.run();
}
