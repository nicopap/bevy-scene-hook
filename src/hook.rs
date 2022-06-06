//! Insert components on loaded scenes.
use bevy::{ecs::system::EntityCommands, ecs::world::EntityRef, prelude::*, scene::SceneInstance};

/// Marker Component for scenes that were succesfully hooked.
#[derive(Component)]
#[non_exhaustive]
pub struct SceneLoaded;

/// Add this as a component to any entity to trigger `hook`'s
/// [`Hook::hook_entity`] method when the scene is loaded.
///
/// You can use it to add your own non-serializable components to entites
/// present in a scene file.
///
/// A typical usage is adding animation or physics collision data to a scene
/// spawned from a file format that do not support that kind of data.
///
/// # Example
///
///  ```rust
/// # use bevy::prelude::{AssetServer, Res, Component, Name, Commands, default};
/// use bevy::ecs::{world::EntityRef, system::EntityCommands};
/// use bevy_scene_hook::{Hook, SceneHook, HookedSceneBundle};
/// # enum PileType { War }
/// # #[derive(Component)] struct Pile(PileType);
/// # #[derive(Component)] struct BirdPupil;
/// fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
///     cmds.spawn_bundle(HookedSceneBundle {
///         scene: asset_server.load("scene.glb#Scene0"),
///         hook: (|entity: &EntityRef, cmds: &mut EntityCommands| {
///             match entity.get::<Name>().map(|t|t.as_str()) {
///                 Some("Pile") => cmds.insert(Pile(PileType::War)),
///                 Some("BirdPupillaSprite") => cmds.insert(BirdPupil),
///                 _ => cmds,
///             };
///         }).into(),
///         ..default()
///     });
/// }
/// ```
#[derive(Component)]
pub struct SceneHook {
    hook: Box<dyn Hook>,
}
impl Default for SceneHook {
    fn default() -> Self {
        Self { hook: Box::new(()) }
    }
}
impl<T: Hook> From<T> for SceneHook {
    fn from(hook: T) -> Self {
        Self::new(hook)
    }
}
impl SceneHook {
    /// Add a hook to a scene, to run for each entities when the scene is
    /// loaded, closures implement `Hook`.
    ///
    ///  # Example
    ///
    ///  ```rust
    /// # use bevy::prelude::{AssetServer, Res, Component, Name, Commands, default};
    /// use bevy::ecs::{world::EntityRef, system::EntityCommands};
    /// use bevy_scene_hook::{Hook, SceneHook, HookedSceneBundle};
    /// # enum PileType { War }
    /// # #[derive(Component)] struct Pile(PileType);
    /// # #[derive(Component)] struct BirdPupil;
    /// fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
    ///     cmds.spawn_bundle(HookedSceneBundle {
    ///         scene: asset_server.load("scene.glb#Scene0"),
    ///         hook: SceneHook::new(|entity: &EntityRef, cmds: &mut EntityCommands| {
    ///             match entity.get::<Name>().map(|t|t.as_str()) {
    ///                 Some("Pile") => cmds.insert(Pile(PileType::War)),
    ///                 Some("BirdPupillaSprite") => cmds.insert(BirdPupil),
    ///                 _ => cmds,
    ///             };
    ///         }),
    ///         ..default()
    ///     });
    /// }
    /// ```
    ///
    ///  You can also implement [`Hook`] on your own types and provide one. Note
    ///  that strictly speaking, you might as well pass a closure. Please check
    ///  the [`Hook`] trait documentation for an example.
    pub fn new<T: Hook>(hook: T) -> Self {
        let hook = Box::new(hook);
        Self { hook }
    }

    /// Same as [`Self::new`] but with type bounds to make it easier to
    /// use a closure.
    ///
    ///  # Example
    ///
    ///  ```rust
    /// # use bevy::prelude::{AssetServer, Res, Component, Name, default, Commands};
    /// use bevy_scene_hook::{Hook, SceneHook, HookedSceneBundle};
    /// # enum PileType { War }
    /// # #[derive(Component)] struct Pile(PileType);
    /// # #[derive(Component)] struct BirdPupil;
    /// fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
    ///     cmds.spawn_bundle(HookedSceneBundle {
    ///         scene: asset_server.load("scene.glb#Scene0"),
    ///         // Notice how the argument types are ellided
    ///         hook: SceneHook::new_fn(|entity, cmds| {
    ///             match entity.get::<Name>().map(|t|t.as_str()) {
    ///                 Some("Pile") => cmds.insert(Pile(PileType::War)),
    ///                 Some("BirdPupillaSprite") => cmds.insert(BirdPupil),
    ///                 _ => cmds,
    ///             };
    ///         }),
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn new_fn<F: Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static>(hook: F) -> Self {
        Self::new(hook)
    }

    /// Add a closure with component parameter as hook.
    ///
    /// This is useful if you only care about a specific component to identify
    /// individual entities of your scene, rather than every possible components.
    ///
    /// See [`Self::with_hook`] for more details.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use bevy::prelude::{AssetServer, Res, Commands, Component, Name, Handle, Scene, default};
    /// use bevy::ecs::system::EntityCommands;
    /// use bevy_scene_hook::{Hook, SceneHook, HookedSceneBundle};
    /// # type DeckData = Scene;
    /// #[derive(Clone)]
    /// struct DeckAssets { player: Handle<DeckData>, oppo: Handle<DeckData> }
    /// #[derive(Component)]
    /// struct Graveyard;
    /// fn hook(decks: &DeckAssets, name: &str, cmds: &mut EntityCommands) {
    ///     match name {
    ///         "PlayerDeck" => cmds.insert(decks.player.clone_weak()),
    ///         "OppoDeck" => cmds.insert(decks.oppo.clone_weak()),
    ///         _ => cmds,
    ///     };
    /// }
    /// fn load_scene(
    ///     mut cmds: Commands,
    ///     decks: Res<DeckAssets>,
    ///     asset_server: Res<AssetServer>,
    /// ) {
    ///     let decks = decks.clone();
    ///     cmds.spawn_bundle(HookedSceneBundle {
    ///         scene: asset_server.load("scene.glb#Scene0"),
    ///         hook: SceneHook::new_comp::<Name, _>(move |name, cmds| hook(&decks, name.as_str(), cmds)),
    ///         ..default()
    ///     }).insert(Graveyard);
    /// }
    /// ```
    pub fn new_comp<C, F>(hook: F) -> Self
    where
        F: Fn(&C, &mut EntityCommands) + Send + Sync + 'static,
        C: Component,
    {
        let hook = move |e: &EntityRef, cmds: &mut EntityCommands| match e.get::<C>() {
            Some(comp) => hook(comp, cmds),
            None => {}
        };
        Self::new(hook)
    }
}

/// Handle adding components to entites named in a loaded scene.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::ecs::{world::EntityRef, system::EntityCommands};
/// use bevy_scene_hook::{Hook, SceneHook, HookedSceneBundle};
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
///     mut commands: Commands,
///     asset_server: Res<AssetServer>,
///     deck_assets: Res<DeckAssets>,
///     cards_assets: Res<CardAssets>,
/// ) {
///     commands.spawn_bundle(HookedSceneBundle {
///         hook: SceneAssets {
///             deck: deck_assets.clone(),
///             cards: cards_assets.clone(),
///         }.into(),
///         scene: asset_server.load("scene.glb#Scene0"),
///         ..default()
///     });
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
    unloaded_instances: Query<(Entity, &SceneInstance, &SceneHook), Without<SceneLoaded>>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut cmds: Commands,
) {
    for (entity, instance, hooked) in unloaded_instances.iter() {
        if let Some(entities) = scene_manager.iter_instance_entities(**instance) {
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
impl Hook for () {
    fn hook_entity(&self, _: &EntityRef, _: &mut EntityCommands) {}
}
