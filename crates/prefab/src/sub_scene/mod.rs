use std::any::TypeId;

use bevy::{prelude::*, ecs::{world::unsafe_world_cell::UnsafeWorldCell, reflect::ReflectCommandExt}, scene::serde::SceneDeserializer, utils::HashSet, reflect::{TypeData, DynamicTypePath}};
use serde::de::DeserializeSeed;

use crate::{component::*, prelude::{ChildrenPrefab, EditorRegistry, EditorRegistryExt, SaveState}};

pub struct SceneUnpackPlugin;

impl Plugin for SceneUnpackPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, SubScenePersistSet::Prepare);
        app.configure_sets(Update, SubScenePersistSet::Unpack);

        app.add_systems(OnEnter(SaveState::Save), (
            (prepare_auto_scene, apply_deferred).chain().before(crate::prelude::serialize_scene),
            clear_after_save.after(crate::prelude::serialize_scene),
        ));
        app.add_systems(PostUpdate, (decompress_scene, apply_deferred, apply_compressed_scenes, apply_deferred).chain());

        app.editor_registry::<CollapsedSubScene>();
        app.editor_registry::<ChildPath>();

        app.register_type::<Vec<usize>>();
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SubScenePersistSet {
    Prepare,
    Unpack,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CollapsedSubScene(pub String);

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct ChildPath(Vec<usize>);


fn clear_after_save(
    mut commands: Commands,
    queue: Query<Entity, With<CollapsedSubScene>>
) {
    for entity in queue.iter() {
        commands.entity(entity).remove::<CollapsedSubScene>();
    }
}

fn prepare_auto_scene(
    world: &mut World
) {
    unsafe {
        let cell = world.as_unsafe_world_cell();

        // Iter all scene roots
        let mut scene_root_query = cell.world_mut().query_filtered::<Entity, With<SceneAutoRoot>>();
        let scene_roots = scene_root_query.iter(cell.world()).collect::<Vec<_>>();

        for root_entity in scene_roots.iter() {

            let registry = cell.world().resource::<crate::prelude::EditorRegistry>().clone();
            let allow_types: Vec<TypeId> = registry
                .registry
                .read()
                .iter()
                .map(|a| a.type_id())
                .collect();

            let mut dyn_scene = DynamicSceneBuilder::from_world(cell.world())
                .allow_all()
                .with_filter(SceneFilter::Allowlist(HashSet::from_iter(
                    allow_types.iter().cloned(),
                )));
            
            
            dyn_scene = recursive_path(&cell, dyn_scene, *root_entity, vec![]);

            let scene = dyn_scene.build();
            let data = scene.serialize_ron(cell.world().resource::<AppTypeRegistry>());

            if let Ok(data) = data {
                info!("serialized sub scene: {:?}", data);
                cell.world_mut().entity_mut(*root_entity).insert(CollapsedSubScene(data));
            } else {
                error!("failed to serialize sub scene: {:?}", data);
            }
        }

    }
}

unsafe fn recursive_path<'w>(cell : &UnsafeWorldCell, scene : DynamicSceneBuilder<'w>, entity: Entity, path: Vec<usize>) -> DynamicSceneBuilder<'w> {
    if let Some(e) = cell.get_entity(entity) {
        cell.world_mut().entity_mut(entity).insert(ChildPath(path.clone()));

        let mut scene = scene.extract_entity(entity);

        if let Some(children) = cell.world().entity(entity).get::<Children>() {
            for (i, child_entity) in children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);

                scene = recursive_path(cell, scene, *child_entity, child_path);
            }
        }
        return scene;
    } else {
        scene
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct DecompressedScene(pub Scene);

fn decompress_scene(
    mut commands: Commands,
    roots: Query<(Entity, &CollapsedSubScene)>,
    type_registry: Res<AppTypeRegistry>,
) {
    for (root_entity, root) in roots.iter() {
        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry.read(),
        };
        let mut deserializer = ron::de::Deserializer::from_str(root.0.as_str()).unwrap();
        let mut dyn_scene : DynamicScene = scene_deserializer
            .deserialize(&mut deserializer)
            .unwrap();

        let mut scene = Scene::from_dynamic_scene(&dyn_scene, &type_registry).unwrap();

        commands
            .entity(root_entity)
            .insert(DecompressedScene(scene))
            .remove::<CollapsedSubScene>();
    }
}


fn apply_compressed_scenes(
    mut commands: Commands,
    mut roots: Query<(Entity, &mut DecompressedScene, &Children)>,
    child_tree: Query<(Entity, Option<&Children>)>,
    editor_registry: Res<EditorRegistry>,
) {
    for (root_entity, mut scene, children) in roots.iter_mut() {
        
        let mut scene_query = scene.world.query::<Entity>();

        let scene_entites = scene_query
            .iter(&scene.world)
            .collect::<Vec<_>>();

        let mut ok = true;
    
        for entity in scene_entites {

            let mut child_path = None;
            if let Some(get_path) = scene.world.entity(entity).get::<ChildPath>() {
                child_path = Some(get_path.clone());
            }

            scene.world.entity_mut(entity).remove::<ChildrenPrefab>();

            if let Some(child_path) = child_path {
                if child_path.0.len() == 0 {
                    continue;
                }

                let mut target_entity = root_entity;
                let mut target_children = Some(children);
                for i in child_path.0.iter() {
                    if let Some(children) = target_children {
                        target_entity = *children.get(*i).unwrap();
                        target_children = child_tree.get(target_entity).unwrap().1;

                    } else {
                        error!("failed to find child path");
                        ok = false;
                        return;
                    }
                }

                if let Some(mut cmds) = commands.get_entity(target_entity) {
                    
                    for clone_fn in editor_registry.clone_components.iter() {
                        (clone_fn.func)(&mut cmds, &scene.world.entity(entity)); 
                    }
                }

                scene.world.entity_mut(entity).despawn();
            } else {
                warn!("failed to find child path in sub entity");
            }
        }

        if ok {
            commands.entity(root_entity).remove::<DecompressedScene>();
        }
    }
}