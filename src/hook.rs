//! Insert components on loaded scenes.

use bevy::{ecs::system::EntityCommands, ecs::world::EntityRef, prelude::*, scene::InstanceId};

/// Marker Component for scenes that were succesfully hooked.
#[derive(Component)]
#[non_exhaustive]
pub struct SceneLoaded;

/// Add this as a component to any entity to trigger `hook`'s
/// [`Hook::hook_entity`] method when the scene is loaded.
#[derive(Component)]
pub(crate) struct SceneHook {
    instance: InstanceId,
    hook: Box<dyn Hook>,
}
impl SceneHook {
    pub(crate) fn new<T: Hook>(instance: InstanceId, hook: T) -> Self {
        let hook = Box::new(hook);
        Self { instance, hook }
    }
    pub(crate) fn new_comp<C, F>(instance: InstanceId, hook: F) -> Self
    where
        F: Fn(&C, &mut EntityCommands) + Send + Sync + 'static,
        C: Component,
    {
        let hook = move |e: &EntityRef, cmds: &mut EntityCommands| match e.get::<C>() {
            Some(comp) => hook(comp, cmds),
            None => {}
        };
        Self::new(instance, hook)
    }
}

/// Handle adding components to entites named in a loaded scene.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::ecs::{world::EntityRef, system::EntityCommands};
/// use bevy_scene_hook::{Hook, HookingSceneSpawner};
/// # type DeckData = Scene;
/// # type Image = Scene;
///
/// #[derive(Component)]
/// struct CardNo(usize);
///
/// #[derive(Clone)]
/// struct DeckAssets { player: Handle<DeckData>, oppo: Handle<DeckData> }
///
/// #[derive(Clone)]
/// struct CardAssets {
///     front_faces: Vec<Handle<Image>>,
///     back_face: Handle<Image>,
/// }
/// struct SceneAssets {
///     deck: DeckAssets,
///     cards: CardAssets,
/// }
/// impl Hook for SceneAssets {
///     fn hook_entity(&self, entity: &EntityRef, cmds: &mut EntityCommands) {
///         match entity.get::<Name>().map(|t|t.as_str()) {
///             Some("PlayerDeck") => cmds.insert(self.deck.player.clone_weak()),
///             Some("OppoDeck") => cmds.insert(self.deck.oppo.clone_weak()),
///             Some("Card") => {
///                 let card_no = entity.get::<CardNo>().unwrap();
///                 cmds.insert(self.cards.front_faces[card_no.0].clone_weak())
///             }
///             _ => cmds,
///         };
///     }
/// }
/// fn load_scene(
///     mut scene_spawner: HookingSceneSpawner,
///     asset_server: Res<AssetServer>,
///     deck_assets: Res<DeckAssets>,
///     cards_assets: Res<CardAssets>,
/// ) {
///     let loader = SceneAssets {
///         deck: deck_assets.clone(),
///         cards: cards_assets.clone(),
///     };
///     scene_spawner.with_hook(asset_server.load("scene.glb#Scene0"), loader);
/// }
/// ```
pub trait Hook: Send + Sync + 'static {
    /// Add [`Component`]s or do anything with entity in the spawned scene
    /// refered by `entity_ref`.
    ///
    /// This runs once for all entities in the spawned scene, once loaded.
    fn hook_entity(&self, entity_ref: &EntityRef, commands: &mut EntityCommands);
}
pub(crate) fn run_hooks(
    unloaded_instances: Query<(Entity, &SceneHook), Without<SceneLoaded>>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut cmds: Commands,
) {
    for (entity, hooked) in unloaded_instances.iter() {
        if let Some(entities) = scene_manager.iter_instance_entities(hooked.instance) {
            for entity_ref in entities.filter_map(|e| world.get_entity(e)) {
                let mut cmd = cmds.entity(entity_ref.id());
                hooked.hook.hook_entity(&entity_ref, &mut cmd);
            }
            cmds.entity(entity).insert(SceneLoaded);
        }
    }
}
impl<F: Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static> Hook for F {
    fn hook_entity(&self, entity_ref: &EntityRef, commands: &mut EntityCommands) {
        (self)(entity_ref, commands)
    }
}
