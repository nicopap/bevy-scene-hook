//! Systems to insert components on loaded scenes.
//!
//! Hooks allow you to run code on a scene after loading it. It provides an API
//! similar to the bevy [`SceneBundle`], but with the ability to add components
//! to scene entities once they are loaded.
//!
//! This crate has two hook types:
//! 1. The very basic [`SceneHook`], it has a `hook` function field. It will
//!    run once per entity within the scene, with the ability to add new
//!    components to the entity and read its existing components.
//! 2. The more advanced [`reload::Hook`]. Which works like [`SceneHook`],
//!    but is aware of **reload** state, and also has access to the ECS `&World`
//!    and the root `Entity` of the scene it is running for.
//!
//! The the respective documentation of [`SceneHook`] and [`reload::Hook`] for
//! usage examples.
mod hook;
pub mod reload;

use bevy::{ecs::system::SystemParam, prelude::*, scene::scene_spawner_system};

pub use hook::{run_hooks, SceneHook, SceneHooked};

#[cfg(doctest)]
#[doc = include_str!("../Readme.md")]
pub struct TestReadme;

/// Bundle a [`SceneHook`] with the standard [`SceneBundle`] components.
///
/// See [`HookedDynamicSceneBundle`] for dynamic scene support.
#[derive(Bundle)]
#[allow(missing_docs /* field description is trivial */)]
pub struct HookedSceneBundle {
    pub hook: SceneHook,
    pub scene: SceneBundle,
}

/// Bundle a [`SceneHook`] with dynamic scenes [`DynamicSceneBundle`] components.
///
/// Similar to [`HookedSceneBundle`], but for dynamic scenes.
#[derive(Bundle)]
#[allow(missing_docs /* field description is trivial */)]
pub struct HookedDynamicSceneBundle {
    pub hook: SceneHook,
    pub scene: DynamicSceneBundle,
}

/// Convenience parameter to query if a scene marked with `M` has been loaded.
#[derive(SystemParam)]
pub struct HookedSceneState<'w, 's, M: Component> {
    query: Query<'w, 's, (), (With<M>, With<SceneHooked>)>,
}
impl<'w, 's, T: Component> HookedSceneState<'w, 's, T> {
    /// Whether any scene with `T` component has been loaded and its hook ran.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.query.iter().next().is_some()
    }
}

/// Convenience run criteria to query if a scene marked with `M` has been loaded.
#[allow(clippy::must_use_candidate)]
pub fn is_scene_hooked<M: Component>(state: HookedSceneState<M>) -> bool {
    state.is_loaded()
}

/// Systems defined in the [`bevy_scene_hook`](crate) crate (this crate).
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum Systems {
    /// System running the hooks.
    SceneHookRunner,
}

/// Plugin to run hooks associated with spawned scenes.
pub struct HookPlugin;
impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SpawnScene,
            run_hooks
                .in_set(Systems::SceneHookRunner)
                .after(scene_spawner_system),
        );
    }
}
