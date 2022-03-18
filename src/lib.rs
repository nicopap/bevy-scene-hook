//! Systems to insert components on loaded scenes
//!
//! Please see the [SceneHook] trait documentation or [the
//! Readme](https://docs.rs/crate/bevy-scene-hook/latest) for detailed
//! usage examples.
use std::marker::PhantomData;

use bevy::{
    ecs::{
        schedule::ShouldRun::{self, No, Yes},
        system::EntityCommands,
    },
    prelude::*,
    scene::InstanceId,
};

/// Add this as a component to any entity to trigger
/// [`<T as SceneHook>::hook`](SceneHook::hook)
#[derive(Component)]
pub struct SceneInstance<T: ?Sized> {
    instance: InstanceId,
    loaded: bool,
    _marker: PhantomData<T>,
}
impl<T: ?Sized> SceneInstance<T> {
    pub fn new(instance: InstanceId) -> Self {
        SceneInstance {
            instance,
            loaded: false,
            _marker: PhantomData,
        }
    }
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

/// Define systems to handle adding components to entites named in a loaded
/// scene.
///
/// Note that you _should_ (but don't need to) use an uninhabited type to
/// `impl` this trait.
///
/// ## Example
///
/// First you need to define your model type:
/// ```rust
/// # use bevy::{prelude::*, ecs::system::EntityCommands};
/// # use bevy_scene_hook::SceneHook;
/// # #[derive(Component)]
/// # enum RigidBody { Dynamic }
///
/// const FINGER_COUNT: usize = 5;
/// #[derive(Component)]
/// struct Finger(usize);
/// // Uninhabited type (there are no values of this type and therefore cannot
/// // be instantiated, since we don't intend to instantiate it, might as well
/// // prevent from doing so)
/// enum HandModel {}
/// impl SceneHook for HandModel {
///     fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {
///         let finger_nodes_names = ["thumb", "index", "major", "ring", "pinky"];
///         let maybe_name_index = finger_nodes_names.iter().enumerate()
///                 .find(|(_, n)| **n == name.as_str())
///                 .map(|(i, _)| i);
///         if let Some(index) = maybe_name_index {
///             cmds.insert_bundle((Finger(index), RigidBody::Dynamic));
///         }
///     }
/// }
/// ```
///
/// Then, you should add the `HandModel::hook` system to your bevy ecs, and can
/// add the `HandModel::when_spawned` run criteria to the systems that rely on
/// the presence of the `Finger` component.
/// ```rust,no_run
/// # use bevy::{prelude::*, ecs::{schedule::ShouldRun, system::EntityCommands}};
/// use bevy_scene_hook::{SceneHook, SceneInstance};
/// # #[derive(Component)]
/// # struct Finger(usize);
/// # enum HandModel {}
/// # impl SceneHook for HandModel {
/// #    fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {}
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
///     app.add_system(HandModel::hook.with_run_criteria(HandModel::when_not_spawned));
/// }
/// ```
///
/// If you have multiple of the same models, you _probably want to use another
/// method_ (and take inspiration from the implementation of this trait). But
/// if you have a known-at-compile-time count of the model (typically for
/// player models) you can use a const generic. In the previous example, it is
/// question of replacing the two lines:
/// ```rust,ignore
/// // From:
/// enum HandModel {}
/// impl SceneHook for HandModel {}
/// // To:
/// enum HandModel<const N: usize> {}
/// impl<const N: usize> ScenefHook for HandModel<N> {}
/// ```
#[allow(unused_parens)]
pub trait SceneHook: Send + Sync + 'static {
    /// Add [`Component`](https://docs.rs/bevy/0.6.1/bevy/ecs/component/trait.Component.html)s
    /// or do anything with `commands`, the
    /// [`EntityCommands`](https://docs.rs/bevy/0.6.1/bevy/ecs/system/struct.EntityCommands.html)
    /// for the named entity in the [`SceneInstance<Self>`] scene.
    fn hook_named_node(name: &Name, commands: &mut EntityCommands);

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
    fn hook(
        mut instance: Query<&mut SceneInstance<Self>>,
        mut cmds: Commands,
        names: Query<&Name>,
        scene_manager: Res<SceneSpawner>,
    ) {
        if let Ok(mut scene_instance) = instance.get_single_mut() {
            if let Some(entities) = scene_manager.iter_instance_entities(scene_instance.instance) {
                for entity in entities {
                    if let Ok(name) = names.get(entity) {
                        Self::hook_named_node(name, &mut cmds.entity(entity));
                    }
                }
                scene_instance.loaded = true;
            }
        }
    }
}
