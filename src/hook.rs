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
/// # Example
///
///  ```rust
/// # use bevy::ecs::{system::Res, component::Component, system::Commands};
/// # use bevy::asset::AssetServer;
/// # use bevy::utils::default;
/// # use bevy::scene::SceneBundle;
/// use bevy_scene_hook::{SceneHook, HookedSceneBundle};
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
///     cmds.spawn(HookedSceneBundle {
///         scene: SceneBundle { scene: asset_server.load("scene.glb#Scene0"), ..default() },
///         hook: SceneHook::new(|entity, cmds, _world| {
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
    hook: Box<dyn Fn(&EntityRef, &mut EntityCommands, &World) + Send + Sync + 'static>,
}
impl SceneHook {
    /// Add a hook to a scene, to run for each entities when the scene is
    /// loaded.
    ///
    /// The hook adds [`Component`]s or do anything with entity in the spawned
    /// scene refered by `EntityRef`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use bevy::ecs::{
    /// #   world::EntityRef, component::Component,
    /// #   system::{Commands, Res, Resource, EntityCommands}
    /// # };
    /// # use bevy::asset::{AssetServer, Handle};
    /// # use bevy::utils::default;
    /// # use bevy::scene::{Scene, SceneBundle};
    /// use bevy_scene_hook::{SceneHook, HookedSceneBundle};
    /// # #[derive(Component)] struct Name;
    /// # type DeckData = Scene;
    /// #[derive(Clone, Resource)]
    /// struct DeckAssets { player: Handle<DeckData>, oppo: Handle<DeckData> }
    ///
    /// fn hook(decks: &DeckAssets, entity: &EntityRef, cmds: &mut EntityCommands, world: &World) {}
    /// fn load_scene(mut cmds: Commands, decks: Res<DeckAssets>, assets: Res<AssetServer>) {
    ///     let decks = decks.clone();
    ///     cmds.spawn(HookedSceneBundle {
    ///         scene: SceneBundle { scene: assets.load("scene.glb#Scene0"), ..default() },
    ///         hook: SceneHook::new(move |entity, cmds, world| hook(&decks, entity, cmds, world)),
    ///     });
    /// }
    /// ```
    pub fn new<F: Fn(&EntityRef, &mut EntityCommands, &World) + Send + Sync + 'static>(
        hook: F,
    ) -> Self {
        Self {
            hook: Box::new(hook),
        }
    }
}

/// Run once [`SceneHook`]s added to [`SceneBundle`](crate::SceneBundle) or
/// [`DynamicSceneBundle`](crate::DynamicSceneBundle) when the scenes are loaded.
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
        for entity_ref in entities.filter_map(|e| world.get_entity(e)) {
            let mut cmd = cmds.entity(entity_ref.id());
            (hooked.hook)(&entity_ref, &mut cmd, world);
        }
    }
}
