

pub mod dock;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(dock::DockPlugin)
            .build()
    }
}