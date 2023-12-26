use bevy::{prelude::*, render::mesh::skinning::SkinnedMesh};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut asset_server : Res<AssetServer>,) {
    commands.spawn(Camera3dBundle::default());

    let mesh_handle = asset_server.load("Pug.gltf#Mesh0/Primitive0");
    let material_handle = asset_server.load("Pug.gltf#Material0");

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            ..default()
        },
        SkinnedMesh::default(),
    ));
}