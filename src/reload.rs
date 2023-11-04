//! Defines reloading [`Hook`]s and supporting system.

use bevy::{
    ecs::{
        system::{Command, EntityCommands},
        world::EntityRef,
    },
    prelude::{
        AssetServer, Bundle, Commands, Component, DespawnRecursiveExt, Entity, IntoSystemConfigs,
        Plugin as BevyPlugin, Query, Res, SceneBundle as BevySceneBundle, World,
    },
    reflect::Reflect,
    scene::{Scene, SceneInstance, SceneSpawner},
};

#[derive(Bundle)]
pub struct SceneBundle {
    pub reload: Hook,
    pub scene: BevySceneBundle,
}

/// A newtype for a dynamic `Fn` that can be run as a hook.
///
/// This is to allow `#[reflect(ignore)]`.
pub struct HookFn(
    pub Box<dyn Fn(&EntityRef, &mut EntityCommands, &World, Entity) + Send + Sync + 'static>,
);

impl Default for HookFn {
    fn default() -> Self {
        Self(Box::new(|_, _, _, _| {}))
    }
}

/// Controls loading and reloading of scenes with a hook.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Reflect)]
pub enum State {
    /// The scene's entites are not yet added to the `World`.
    Loading,
    /// The scene's entities are now in the `World` and its entities have the
    /// components added by the scene's [`Hook::hook`].
    Hooked,
    /// The scene's entities, whether they are its direct children or were
    /// unparented are to be despawned next time [`run_hooks`] runs, to be
    /// reloaded, running [`Hook::hook`] again.
    ///
    /// The spawned scene is loaded using [`Hook::file_path`].
    MustReload,
    /// The scene's entities, whether they are its direct children or were
    /// unparented are to be despawned next time [`run_hooks`] runs, the scene
    /// entity itself will also be deleted.
    MustDelete,
}
/// A variant of [`crate::SceneHook`] that allows for reloading.
///
/// Please read [`crate::SceneHook`]'s documentation for more details on how
/// hooking works.
///
/// ## Warnings
///
/// The despawning and respawning will generate a lot of warnings, because it
/// despawns entities in the scene recursively.
///
/// Entities are despawning recursively, because you might have added
/// children to entities in a scene, and we want to eliminate them when
/// reloading. But oftentimes, in a scene, entities form a hierarchy, thus,
/// we ask bevy to despawn several times the same entity, resulting in a
/// warning.
#[derive(Component, Reflect)]
pub struct Hook {
    /// File path to scene, used when [`State::MustReload`].
    pub file_path: String,
    /// The reload state of the scene, see type's doc.
    pub state: State,
    /// The hook ran on each entity in the scene when spawned and respawned.
    ///
    /// - [`& EntityRef`]: A reference to the current node in the scene, you can use
    ///   it to query for existing components, useful to get the name of the entity.
    /// - [`&mut EntityCommands`]: Add/remove components to the current entity.
    /// - [`& World`]: The world
    /// - [`Entity`]: The `Entity` of the scene this entity is part of. May be useful
    ///   in combination with `&World` to get components of the scene.
    #[reflect(ignore)]
    pub hook: HookFn,
}
impl Hook {
    pub fn new<F>(hook: F, file_path: String) -> Self
    where
        F: Fn(&EntityRef, &mut EntityCommands, &World, Entity) + Send + Sync + 'static,
    {
        Hook {
            state: State::Loading,
            file_path,
            hook: HookFn(Box::new(hook)),
        }
    }
}
/// Command to update [`Hook`] in a [`Commands`] context.
struct UpdateHook {
    entity: Entity,
    new_state: State,
}
impl Command for UpdateHook {
    fn apply(self, world: &mut World) {
        if let Some(mut hook) = world.get_mut::<Hook>(self.entity) {
            hook.state = self.new_state;
        }
    }
}

/// Run [`Hook`]s and respawn scenes according to [`Hook::state`].
pub fn run_reloadable_hooks(
    instances: Query<(Entity, &SceneInstance, &Hook)>,
    scene_manager: Res<SceneSpawner>,
    assets: Res<AssetServer>,
    world: &World,
    mut cmds: Commands,
) {
    use State::*;
    for (entity, instance, reload) in instances.iter() {
        let instance_ready = scene_manager.instance_is_ready(**instance);
        match reload.state {
            Loading if instance_ready => {
                cmds.add(UpdateHook {
                    entity,
                    new_state: Hooked,
                });
                let entities = scene_manager.iter_instance_entities(**instance);
                for entity_ref in entities.filter_map(|e| world.get_entity(e)) {
                    let mut cmd = cmds.entity(entity_ref.id());
                    (reload.hook.0)(&entity_ref, &mut cmd, world, entity);
                }
            }
            Hooked | Loading => continue,
            MustReload => {
                let entities = scene_manager.iter_instance_entities(**instance);
                for entity in entities.filter(|e| world.get_entity(*e).is_some()) {
                    cmds.entity(entity).despawn_recursive();
                }
                cmds.add(UpdateHook {
                    entity,
                    new_state: Loading,
                });
                cmds.entity(entity)
                    .insert(assets.load::<Scene>(&reload.file_path))
                    .remove::<SceneInstance>();
            }
            MustDelete => {
                let entities = scene_manager.iter_instance_entities(**instance);
                for entity in entities.filter(|e| world.get_entity(*e).is_some()) {
                    cmds.entity(entity).despawn_recursive();
                }
                cmds.entity(entity).despawn_recursive();
            }
        }
    }
}

/// The plugin to manage reloading [`Hook`]s. It just registers [`Hook`],
/// [`State`] and adds the [`run_hooks`] system.
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Hook>()
            .register_type::<State>()
            .add_systems(
                bevy::prelude::SpawnScene,
                run_reloadable_hooks.after(bevy::scene::scene_spawner_system),
            );
    }
}
