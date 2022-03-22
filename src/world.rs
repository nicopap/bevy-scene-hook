//! `SceneHook` with access to the bevy `World`.
//!
//! Please see the [`SceneHook`] trait documentation or [the
//! Readme](https://docs.rs/crate/bevy-scene-hook/latest) for detailed
//! usage examples.
use bevy::{
    ecs::schedule::ShouldRun::{self, No, Yes},
    prelude::*,
};

use crate::SceneInstance;

/// Define systems to handle adding components to entites named in a loaded
/// scene, with access to the bevy [`World`](https://docs.rs/bevy/0.6.1/bevy/ecs/world/struct.World.html).
///
/// Very similar to [`crate::SceneHook`], but let you access the bevy world in
/// the [`SceneHook::hook_named_node`] method. The downside is that the
/// [`SceneHook::hook`] system must be added to the app as an exclusive system.
/// Though, since it will only run once, it's not too much of a deal.
///
/// See the [`SceneHook::hook_named_node`] documentation for more details.
///
/// World access can be useful if, for example, you need access to an `Assets`
/// or do just generaly anything.
///
/// The impact on performance is "meh" since the system is only ran once, it
/// won't be much of a bottleneck.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy::{prelude::*, ecs::{schedule::ShouldRun, system::EntityCommands}};
/// // note that we use   vvvvvvvvvvvvvvvv
/// use bevy_scene_hook::{world::SceneHook, SceneInstance};
/// # #[derive(Component)]
/// # struct Finger(usize);
/// # enum HandModel {}
/// # impl SceneHook for HandModel {
/// #    fn hook_named_node(name: Name, world: &mut World, entity: Entity) {}
/// # }
/// fn play_piano(fingers: Query<&Finger>) {}
/// fn move_fingers(fingers: Query<&mut Finger>) {}
/// fn main() {
///     let mut app = App::new();
///     app.add_system_set(
///         SystemSet::new()
///             .with_system(play_piano)
///             .with_system(move_fingers)
///             // Systems that use a `Finger` component can be made to run
///             // only when the model is spawned with this run criteria
///             .with_run_criteria(HandModel::when_spawned),
///     );
///     // You need to add the `HandModel::hook` system with the
///     // `when_not_spawned` run criteria
///     app.add_system(
///         HandModel::hook
///         // with the world version, the hook needs to be added as an exclusive_system
///         //  vvvvvvvvvvvvvvvvvvv
///             .exclusive_system()
///             .with_run_criteria(HandModel::when_not_spawned),
///     );
/// }
/// ```
#[allow(unused_parens)]
pub trait SceneHook: Send + Sync + 'static {
    /// Add [`Component`](https://docs.rs/bevy/0.6.1/bevy/ecs/component/trait.Component.html)s
    /// or do anything with the bevy [`World`](https://docs.rs/bevy/0.6.1/bevy/ecs/world/struct.World.html),
    /// once for each `Entity` added to the world through the scene.
    ///
    /// You can add components to the entity with the following code:
    /// ```rust
    /// # use bevy::prelude::{Component, Entity, World, Name};
    /// #[derive(Component)]
    /// struct MyComponent(usize);
    /// # struct Foo;
    /// # impl Foo {
    ///     fn hook_named_node(name: Name, world: &mut World, entity: Entity) {
    ///         world.entity_mut(entity).insert(MyComponent(10));
    ///     }
    /// # }
    /// ```
    ///
    /// # Considerations
    ///
    /// Note that the `Entity` _should_ be available, but the nature of
    /// complete unfettered access to a `&mut World` means you can shoot
    /// yourself in the foot by, for example, despawning an entity that was
    /// supposed to be in the scene. So please don't do that :)
    ///
    /// # Panics
    ///
    /// * If you remove the `SceneInstance<Self>` entity from the ECS.
    fn hook_named_node(name: Name, world: &mut World, entity: Entity);

    /// [`RunCriteria`](https://docs.rs/bevy/0.6.1/bevy/ecs/prelude/struct.RunCriteria.html)
    /// to add to systems that only run after the scene was hooked.
    fn when_spawned(instance: Query<&SceneInstance<Self>>) -> ShouldRun {
        let is_loaded = instance.get_single().map_or(false, |inst| inst.loaded);
        (if is_loaded { Yes } else { No })
    }
    /// [`RunCriteria`](https://docs.rs/bevy/0.6.1/bevy/ecs/prelude/struct.RunCriteria.html)
    /// to add to systems that only run before the scene was hooked.
    fn when_not_spawned(instance: Query<&SceneInstance<Self>>) -> ShouldRun {
        let is_loaded = instance.get_single().map_or(false, |inst| inst.loaded);
        (if !is_loaded { Yes } else { No })
    }

    /// Calls [`Self::hook_named_node`] for each named entity in the scene
    /// specified in [`SceneInstance<Self>`].
    ///
    /// Add this system to your app for the hooks to run.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy::{prelude::*, ecs::{schedule::ShouldRun, system::EntityCommands}};
    /// use bevy_scene_hook::world::SceneHook;
    /// # struct Finger(usize);
    /// # enum HandModel {}
    /// # impl SceneHook for HandModel {
    /// #    fn hook_named_node(name: Name, world: &mut World, entity: Entity) {}
    /// # }
    /// fn main() {
    ///     let mut app = App::new();
    ///     app.add_system(
    ///         HandModel::hook
    ///         // with the world version, the hook needs to be added as an exclusive_system
    ///         //  vvvvvvvvvvvvvvvvvvv
    ///             .exclusive_system()
    ///             .with_run_criteria(HandModel::when_not_spawned),
    ///     );
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// * If there is no `Res<SceneSpawner>` available (aka you didn't add the
    ///   `DefaultPlugins` to the app).
    /// * If you somehow manage to remove the `SceneInstance<Self>` entity from
    ///   the ECS in [`SceneHook::hook_named_node`]
    fn hook(world: &mut World) {
        let get_instance_mut =
            |world: &mut World| world.query_filtered::<&mut SceneInstance<Self>, Without<Name>>();
        if let Some(instance) = get_instance_mut(world).iter_mut(world).next() {
            let instance_id = instance.instance;
            // Spawner resource is defined in the default bevy plugin
            let spawner = world.get_resource::<SceneSpawner>().unwrap();
            let entities = spawner
                .iter_instance_entities(instance_id)
                .map(Iterator::collect::<Vec<_>>);
            if let Some(entities) = entities {
                let mut names = world.query::<&Name>();
                for entity in entities {
                    if let Ok(name) = names.get(world, entity) {
                        Self::hook_named_node(name.clone(), world, entity);
                    }
                }
                // The `SceneInstance` existence was tested a bit earlier, the
                // only way it's not there anymore is if the user removed it,
                // and they have been warned against it.
                let mut scene_instance = get_instance_mut(world).iter_mut(world).next().unwrap();
                scene_instance.loaded = true;
            }
        }
    }
}
