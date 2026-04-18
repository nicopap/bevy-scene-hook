//! Systems to insert components on loaded scenes.
//!
//! Please see the [`SceneHook`] documentation for detailed examples.

use bevy::ecs::{
    component::Component,
    entity::Entity,
    prelude::{Without, World},
    system::{Commands, EntityCommands, Query, Res},
    world::EntityRef,
};
use bevy::scene::{SceneInstance, SceneSpawner};

/// Marker Component for scenes that were hooked.
#[derive(Component, Debug)]
#[non_exhaustive]
pub struct SceneHooked;

/// Add this as a component to any entity to run `hook`
/// when the scene is loaded.
///
/// You can use it to add your own non-serializable components to entites
/// present in a scene file.
///
/// A typical usage is adding animation, physics collision data or marker
/// components to a scene spawned from a file format that do not support it.
///
/// # Access to `World`
///
/// A variant of `SceneHook` exists with access to the scene `Entity` and the `&World`,
/// check [`crate::reload::Hook`] if you need such features.
///
/// # Example
///
///  ```rust
/// # use bevy::ecs::{system::Res, component::Component, system::Commands};
/// # use bevy::asset::AssetServer;
/// # use bevy::utils::default;
/// # use bevy::scene::SceneRoot;
/// use bevy_scene_hook::{SceneHook, HookedSceneRoot};
/// # #[derive(Component)]
/// # struct Name; impl Name { fn as_str(&self) -> &str { todo!() } }
/// enum PileType { Drawing }
///
/// #[derive(Component)]
/// struct Pile(PileType);
///
/// #[derive(Component)]
/// struct Card;
///
/// fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
///     cmds.spawn(HookedSceneRoot {
///         scene: SceneRoot(asset_server.load("scene.glb#Scene0")),
///         hook: SceneHook::new(|entity, cmds| {
///             match entity.get::<Name>().map(|t|t.as_str()) {
///                 Some("Pile") => cmds.insert(Pile(PileType::Drawing)),
///                 Some("Card") => cmds.insert(Card),
///                 _ => cmds,
///             };
///         }),
///     });
/// }
/// ```
#[derive(Component)]
pub struct SceneHook {
    hook: Box<dyn Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static>,
}
impl SceneHook {
    /// Add a hook to a scene, to run for each entities when the scene is
    /// loaded.
    ///
    /// The hook adds [`Component`]s or do anything with entity in the spawned
    /// scene refered by `EntityRef`.
    ///
    /// # Access to `World`
    ///
    /// A variant of `SceneHook` exists with access to the scene `Entity` and the `&World`,
    /// check [`crate::reload::Hook`] if you need such features.
    ///
/// # Example
    ///
    /// ```rust
    /// # use bevy::ecs::{
    /// #   world::EntityRef, component::Component,
    /// #   system::{Commands, Res, EntityCommands}
    /// # };
    /// # use bevy::asset::AssetServer;
    /// # use bevy::scene::SceneRoot;
    /// use bevy_scene_hook::{SceneHook, HookedSceneRoot};
    /// # #[derive(Component)] struct Name;
    ///
    /// fn hook(entity: &EntityRef, cmds: &mut EntityCommands) {
    ///     let name = entity.get::<Name>();
    ///     let _ = name;
    /// }
    /// fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
    ///     cmds.spawn(HookedSceneRoot {
    ///         scene: SceneRoot(asset_server.load("scene.glb#Scene0")),
    ///         hook: SceneHook::new(hook),
    ///     });
    /// }
    /// ```
    pub fn new<F: Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static>(hook: F) -> Self {
        Self { hook: Box::new(hook) }
    }
}

/// Run once [`SceneHook`]s added to [`SceneRoot`](crate::SceneRoot) or
/// [`DynamicSceneRoot`](crate::DynamicSceneRoot) when the scenes are loaded.
pub fn run_hooks(
    unloaded_instances: Query<(Entity, &SceneInstance, &SceneHook), Without<SceneHooked>>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut cmds: Commands,
) {
    for (entity, instance, hooked) in unloaded_instances.iter() {
        if scene_manager.instance_is_ready(**instance) {
            cmds.entity(entity).insert(SceneHooked);
        }
        let entities = scene_manager
            .iter_instance_entities(**instance)
            .chain(std::iter::once(entity));
        for entity_ref in entities.filter_map(|e| world.get_entity(e).ok()) {
            let mut cmd = cmds.entity(entity_ref.id());
            (hooked.hook)(&entity_ref, &mut cmd);
        }
    }
}
