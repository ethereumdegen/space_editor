

pub mod dock;

pub mod panels;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(dock::DockPlugin)
            .build()
    }
}

pub fn spawn_editor_ui(
    mut commands : Commands
) {

}