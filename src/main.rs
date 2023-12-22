use bevy::{prelude::*, render::{RenderPlugin, settings::{RenderCreation, WgpuSettings, WgpuFeatures}}, pbr::wireframe::WireframePlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "space_editor".to_string(), // ToDo
            // Bind to canvas included in `index.html`
            canvas: Some("#bevy".to_owned()),
            // The canvas size is constrained in index.html and build/web/styles.css
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5 and Ctrl+R
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "editor")]
    {
        use space_editor::SpaceEditorPlugin;
        use space_editor_ui::simple_editor_setup;
        app.add_plugins(SpaceEditorPlugin)
            .add_systems(Startup, simple_editor_setup);
    }
    app.run();
}
