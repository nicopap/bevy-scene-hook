//! Systems to insert components on loaded scenes.
//!
//! Please see the [`HookingSceneSpawner`] documentation for detailed examples.
use bevy::{
    ecs::{
        schedule::ShouldRun,
        system::{EntityCommands, SystemParam},
    },
    prelude::*,
    scene::InstanceId,
};
mod hook;

use hook::SceneHook;
pub use hook::{run_hooks, Hook, SceneLoaded};

/// Convenience parameter to query if a scene marked with `M` has been loaded.
///
/// # Example
///
/// ```rust, no_run
/// # use bevy::prelude::*;
/// use bevy_scene_hook::{Hook, HookPlugin, HookingSceneSpawner, HookedSceneState};
///
/// #[derive(Component)]
/// struct Graveyard;
///
/// #[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
/// pub enum GameState { Loading, Playing }
///
/// fn load_scene(
///     mut scene_spawner: HookingSceneSpawner,
///     mut cmds: Commands,
///     asset_server: Res<AssetServer>,
/// ) {
///     let res = scene_spawner.with_comp_hook(
///         asset_server.load("scene.glb#Scene0"),
///         |name: &Name, cmds| {
///             match name.as_str() {
///                 "OppoCardSpawn" => cmds.insert(GlobalTransform::default()),
///                 _ => cmds,
///             };
///         },
///     );
///     cmds.entity(res.entity).insert(Graveyard);
/// }
///
/// fn complete_load_screen(
///     mut state: ResMut<State<GameState>>,
///     scene: HookedSceneState<Graveyard>,
/// ) {
///     if scene.is_loaded() {
///         state.set(GameState::Playing).expect("no state issues");
///     }
/// }
/// fn main() {
///     let mut app = App::new();
///     app
///         .add_plugin(HookPlugin)
///         .add_state(GameState::Loading)
///         .add_startup_system(load_scene)
///         .add_system_set(
///             SystemSet::on_update(GameState::Loading)
///                 .with_system(complete_load_screen),
///         );
/// }
/// ```
#[derive(SystemParam)]
pub struct HookedSceneState<'w, 's, M: Component> {
    query: Query<'w, 's, (), (With<M>, With<SceneLoaded>)>,
}
impl<'w, 's, T: Component> HookedSceneState<'w, 's, T> {
    pub fn is_loaded(&self) -> bool {
        self.query.iter().next().is_some()
    }
}
/// Convenience run criteria to query if a scene marked with `M` has been loaded.
///
/// # Example
///
/// ```rust, no_run
/// # use bevy::prelude::*;
/// use bevy_scene_hook::{Hook, HookingSceneSpawner, is_scene_hooked};
///
/// #[derive(Component)] struct OppoCardSpawner;
/// #[derive(Component)] struct PlayerCardSpawner;
///
/// #[derive(Component)]
/// struct Graveyard;
///
/// fn load_scene(
///     mut scene_spawner: HookingSceneSpawner,
///     mut cmds: Commands,
///     asset_server: Res<AssetServer>,
/// ) {
///     let res = scene_spawner.with_comp_hook(
///         asset_server.load("scene.glb#Scene0"),
///         |name: &Name, cmds| {
///             match name.as_str() {
///                 "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
///                 "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
///                 _ => cmds,
///             };
///         },
///     );
///     cmds.entity(res.entity).insert(Graveyard);
/// }
/// fn use_spawner(q: Query<&GlobalTransform, With<PlayerCardSpawner>>) {
///     let transform = q.single();
///     // do stuff with transform
/// }
/// fn main() {
///     let mut app = App::new();
///     app
///         .add_startup_system(load_scene)
///         .add_system(use_spawner.with_run_criteria(is_scene_hooked::<Graveyard>));
/// }
/// ```
pub fn is_scene_hooked<M: Component>(state: HookedSceneState<M>) -> ShouldRun {
    match state.is_loaded() {
        true => ShouldRun::Yes,
        false => ShouldRun::No,
    }
}

/// Return value of [`HookingSceneSpawner`] methods.
#[derive(Debug, Copy, Clone)]
pub struct HookResult {
    /// The entity to which the hook has been added.
    ///
    /// If using [`HookingSceneSpawner::child_comp_hook`] or
    /// [`HookingSceneSpawner::child`], this is the `parent` argument.
    pub entity: Entity,
    /// The instance of the spawned scene.
    pub instance: InstanceId,
}

/// Parameter to add scenes that run arbitrary hooks when the scene is fully
/// loaded.
///
/// You can use it to add your own non-serializable components to entites
/// present in a scene file.
///
/// A typical usage is adding animation or physics collision data to a scene
/// spawned from a file format that do not support that kind of data.
///
/// # Example
///
/// The most basic usage relies on a closure. Here, we capture the `decks`
/// value so that we can use it in the closure.
///
/// ```rust
/// # use bevy::prelude::{AssetServer, Res, Commands, Component, Name, Handle, Scene};
/// use bevy::ecs::system::EntityCommands;
/// use bevy_scene_hook::{Hook, HookingSceneSpawner};
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
///     mut scene_spawner: HookingSceneSpawner,
///     mut cmds: Commands,
///     decks: Res<DeckAssets>,
///     asset_server: Res<AssetServer>,
/// ) {
///     let decks = decks.clone();
///     let res = scene_spawner.with_comp_hook(
///         asset_server.load("scene.glb#Scene0"),
///         move |name: &Name, cmds| hook(&decks, name.as_str(), cmds),
///     );
///     cmds.entity(res.entity).insert(Graveyard);
/// }
/// ```
#[derive(SystemParam)]
pub struct HookingSceneSpawner<'w, 's> {
    cmds: Commands<'w, 's>,
    scene_spawner: ResMut<'w, SceneSpawner>,
}
impl<'w, 's> HookingSceneSpawner<'w, 's> {
    /// Add a hook to a scene, to run for each entities when the scene is
    /// loaded, closures implement `Hook`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use bevy::prelude::{AssetServer, Res, Component, Name};
    /// use bevy::ecs::{world::EntityRef, system::EntityCommands};
    /// use bevy_scene_hook::{Hook, HookingSceneSpawner};
    /// # enum PileType { War }
    /// # #[derive(Component)] struct Pile(PileType);
    /// # #[derive(Component)] struct BirdPupil;
    /// fn load_scene(
    ///     mut scene_spawner: HookingSceneSpawner,
    ///     asset_server: Res<AssetServer>,
    /// ) {
    ///     scene_spawner.with_hook(
    ///         asset_server.load("scene.glb#Scene0"),
    ///         |entity: &EntityRef, cmds: &mut EntityCommands| {
    ///             match entity.get::<Name>().map(|t|t.as_str()) {
    ///                 Some("Pile") => cmds.insert(Pile(PileType::War)),
    ///                 Some("BirdPupillaSprite") => cmds.insert(BirdPupil),
    ///                 _ => cmds,
    ///             };
    ///         },
    ///     );
    /// }
    /// ```
    ///
    /// You can also implement [`Hook`] on your own types and provide one. Note
    /// that strictly speaking, you might as well pass a closure. Please check
    /// the [`Hook`] trait documentation for an example.
    pub fn with_hook<T: Hook>(&mut self, handle: Handle<Scene>, hook: T) -> HookResult {
        let instance = self.scene_spawner.spawn(handle);
        let hook = SceneHook::new(instance, hook);
        let entity = self.cmds.spawn().insert(hook).id();
        HookResult { entity, instance }
    }

    /// Add a hook to a scene, to run for each entities when the scene is
    /// loaded, and spawn the scene as a child of `parent`.
    ///
    /// See [`Self::with_hook`] for more details.
    pub fn child<T: Hook>(&mut self, handle: Handle<Scene>, parent: Entity, hook: T) -> HookResult {
        let instance = self.scene_spawner.spawn_as_child(handle, parent);
        let hook = SceneHook::new(instance, hook);
        let entity = self.cmds.entity(parent).insert(hook).id();
        // TODO: remove
        assert_eq!(entity, parent);
        HookResult { entity, instance }
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
    /// The original version of this library, relied exclusively on the `Name`
    /// parameter. Here is how you can emulate it.
    /// ```rust
    /// # use bevy::prelude::{AssetServer, Res, Component, Name};
    /// use bevy_scene_hook::HookingSceneSpawner;
    /// # #[derive(Component)] struct OppoCardSpawner;
    /// # #[derive(Component)] struct PlayerCardSpawner;
    /// fn load_scene(
    ///     mut scene_spawner: HookingSceneSpawner,
    ///     asset_server: Res<AssetServer>,
    /// ) {
    ///     scene_spawner.with_comp_hook(
    ///         asset_server.load("scene.glb#Scene0"),
    ///         |name: &Name, cmds| {
    ///             match name.as_str() {
    ///                 "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
    ///                 "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
    ///                 _ => cmds,
    ///             };
    ///         },
    ///     );
    /// }
    /// ```
    ///
    pub fn with_comp_hook<C, F>(&mut self, handle: Handle<Scene>, hook: F) -> HookResult
    where
        F: Fn(&C, &mut EntityCommands) + Send + Sync + 'static,
        C: Component,
    {
        let instance = self.scene_spawner.spawn(handle);
        let hook = SceneHook::new_comp(instance, hook);
        let entity = self.cmds.spawn().insert(hook).id();
        HookResult { entity, instance }
    }

    /// Add a closure with component parameter as hook, and spawn the scene as a child of `parent`.
    ///
    /// See [`Self::with_hook`] and [`Self::child`] for more details.
    pub fn child_comp_hook<C, F>(
        &mut self,
        handle: Handle<Scene>,
        parent: Entity,
        hook: F,
    ) -> HookResult
    where
        F: Fn(&C, &mut EntityCommands) + Send + Sync + 'static,
        C: Component,
    {
        let instance = self.scene_spawner.spawn_as_child(handle, parent);
        let hook = SceneHook::new_comp(instance, hook);
        let entity = self.cmds.entity(parent).insert(hook).id();
        // TODO: remove
        assert_eq!(entity, parent);
        HookResult { entity, instance }
    }
}

/// Systems defined in the [`bevy_scene_hook`](crate) crate (this crate).
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum Systems {
    /// System running the hooks registered with [`HookingSceneSpawner`].
    SceneHookRunner,
}

/// Plugin to run hooks associated with spawned scenes.
pub struct HookPlugin;
impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(run_hooks.label(Systems::SceneHookRunner));
    }
}
