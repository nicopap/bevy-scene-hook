# Bevy Scene hook

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/bevy_scene_hook.svg)](https://crates.io/crates/bevy_scene_hook)
[![Apache 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/bevy-scene-hook/badge.svg)](https://docs.rs/bevy-scene-hook/)

A proof of concept for adding components ad-hoc within code to entities
spawned through scenes (such as `gltf` files) in the [bevy game engine].

If you don't mind adding such a small dependency to your code rather than
copy/pasting the code as a module, you can get it from [crates.io].

## Usage

1. Add the crate to your dependencies
```toml
[dependencies]
bevy-scene-hook = "5.1"
```
2. Add the plugin
```rust
.add_plugin(HookPlugin)
```

### Example

 ```rust
use bevy_scene_hook::{SceneHook, HookedSceneBundle};

enum PileType { Drawing }

#[derive(Component)]
struct Pile(PileType);

#[derive(Component)]
struct Card;

fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn_bundle(HookedSceneBundle {
        scene: SceneBundle { scene: asset_server.load("scene.glb#Scene0"), ..default() },
        hook: SceneHook::new(|entity, cmds| {
            match entity.get::<Name>().map(|t|t.as_str()) {
                Some("Pile") => cmds.insert(Pile(PileType::Drawing)),
                Some("Card") => cmds.insert(Card),
                _ => cmds,
            };
        }),
    });
}
```

It loads the `scene.glb` file when the game starts. When the scene is fully loaded,
the closure passed to `SceneHook::new` is ran for each entity
present in the scene. We add a `Pile` component to entities
with a `Name` component of value `"Pile"`.

It is possible to name object in `glb` scenes in blender using the Outliner
dock (the tree view at the top right) and double-clicking object names.

### Implementation

`bevy-scene-hook` is a tinny crate, here is copy/pastable code you can directly vendor
in your project:

```rust
use bevy::{
    prelude::*,
    scene::SceneInstance,
    ecs::{world::EntityRef, system::EntityCommands},
};
#[derive(Component, Debug)]
pub struct SceneHooked;

#[derive(Component)]
pub struct SceneHook {
    hook: Box<dyn Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static>,
}
impl SceneHook {
    pub fn new<F: Fn(&EntityRef, &mut EntityCommands) + Send + Sync + 'static>(hook: F) -> Self {
        Self { hook: Box::new(hook) }
    }
}

pub fn run_hooks(
    unloaded_instances: Query<(Entity, &SceneInstance, &SceneHook), Without<SceneHooked>>,
    scene_manager: Res<SceneSpawner>,
    world: &World,
    mut cmds: Commands,
) {
    for (entity, instance, hooked) in unloaded_instances.iter() {
        let entities = scene_manager.iter_instance_entities(**instance);
        for entity_ref in entities.filter_map(|e| world.get_entity(e)) {
            let mut cmd = cmds.entity(entity_ref.id());
            (hooked.hook)(&entity_ref, &mut cmd);
        }
        cmds.entity(entity).insert(SceneHooked);
    }
}

pub struct HookPlugin;
impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) { app.add_system(run_hooks); }
}
```

Note that `bevy-scene-hook` also has a few items defined for user convinience:

- `HookedSceneBundle`
- `HookedSceneState`
- `is_scene_hooked`

Those extra items are all defined in `lib.rs`.

[bevy game engine]: https://bevyengine.org/
[crates.io]: https://crates.io/crates/bevy-scene-hook
[warlock-source]: https://github.com/team-plover/warlocks-gambit

## Change log

* `1.1.0`: Add `is_loaded` method to `SceneInstance`
* `1.2.0`: Add the `world` module containing a `SceneHook` trait that has
  exclusive world access. Useful if you want access to assets for example.
* `2.0.0`: **Breaking**: bump bevy version to `0.7` (you should be able to
  upgrade from `1.2.0` without changing your code)
* `3.0.0`: **Breaking**: completely rework the crate.
    * Remove the `world` module, as the base hook method has become much more
      powerful.
    * Rename `SceneHook` to `Hook`, now `Hook` has a unique method to implement.
    * You don't have to add any system yourself, now you have to add the
      `HookPlugin` plugin to your app.
    * Move the API exclusively to a new `SystemParam`: `HookingSceneSpawner`.
      Please use that parameter to add and remove scenes that contain hooks.
      (please tell if you you accidentally spell it `HonkingSceneSpawner` more
      than once :duck:)
    * Moved the `when_spawned` run criteria to the `is_scene_hooked`
      function exposed at the root of the crate, the `HookedSceneState`
      system parameter or the `SceneLoaded` component. Please use any of
      those three instead of `when_spawned`.
    * Now supports passing closures as hooks, instead of having to define
      a trait each time.
    * Now supports adding multiple of the same scene! Doesn't handle
      hot-reloading, but that's alright since bevy's scene hot-reloading
      is currently broken anyway :D
* `3.1.0`: make `run_hooks` system public so that it's possible to add it to
  any stage you want in relation to any other system you want.
* `4.0.0`: **Breaking**: bump bevy version to `0.8`
    * Uses the new scene bundle system
    * Rename `SceneLoaded` to `SceneHooked`.
    * Removed the `Hook` trait, now `SceneHook::new` accepts a closure.
    * Instead of using `HookingSceneSpawner`, uses `HookedSceneBundle`
      and spawn it into an entity.
* `4.1.0`: Add `HookedDynamicSceneBundle` to use with `DynamicScene`s.
  Thanks Shatur (#3)
* `5.0.0`: **Breaking**: bump bevy version to `0.9`
* `5.1.1`: My bad, I accidentally published to version `5.1.0` instead of
  `5.0.0`

### Version matrix

| bevy | latest supporting version      |
|------|--------|
| 0.9  | 5.1.1 |
| 0.8  | 4.1.0 |
| 0.7  | 3.1.0 |
| 0.6  | 1.2.0 |


## License

This library is licensed under Apache 2.0.
