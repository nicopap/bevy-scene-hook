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

```toml
[dependencies]
bevy-scene-hook = "4.0"
```

The following snippet of code is extracted from
[Warlock's Gambit source code][warlock-source].

It loads the `scene.glb` file when the game starts. When the scene is fully loaded,
the closure passed to `with_comp_hook` is ran for each entity with a `Name` component
present in the scene. We add a `Graveyard` component to the `Entity` containing the
`SceneHook`, so that it can later be checked for whether the scene completed loading.

It is possible to name object in `glb` scenes in blender using the Outliner
dock (the tree view at the top right) and double-clicking object names.

```rust
#[derive(Component)]
pub struct Graveyard;

fn hook(
    decks: &DeckAssets,
    name: &str,
    cmds: &mut EntityCommands,
) {
    match name {
        "PlayerPerspective_Orientation" => cmds.insert_bundle((
            RayCastSource::<HandRaycast>::new(),
            RayCastSource::<SleeveArea>::new(),
            RayCastSource::<HandDisengageArea>::new(),
        )),
        "PlayerDeck" => cmds.insert(decks.player.clone_weak()),
        "OppoDeck" => cmds.insert(decks.oppo.clone_weak()),
        "PlayerPile" => cmds.insert(Pile::new(Participant::Player)),
        "OppoPile" => cmds.insert(Pile::new(Participant::Oppo)),
        "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
        "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
        "PlayerHand" => cmds.insert_bundle((PlayerHand, Animated::bob(2.0, 0.05, 7.0))),
        "OppoHand" => cmds.insert_bundle((OppoHand, Animated::bob(1.0, 0.3, 6.0))),
        "Pile" => cmds.insert(Pile::new(PileType::War)),
        "ManBody" => cmds.insert(Animated::breath(0.0, 0.03, 6.0)),
        "ManHead" => cmds.insert(Animated::bob(6. / 4., 0.1, 6.0)),
        "Bird" => cmds.insert(Animated::breath(0.0, 0.075, 5.0)),
        "BirdPupillaSprite" => cmds.insert(BirdPupil),
        "BirdEyePupilla" => cmds.insert_bundle((BirdPupilRoot, Animated::bob(5. / 4., 0.02, 5.0))),
        _ => cmds,
    };
}
fn load_scene(
    mut cmds: Commands,
    decks: Res<DeckAssets>,
    asset_server: Res<AssetServer>,
) {
    let decks = decks.clone();
    cmds.insert(Graveyard).spawn_bundle(HookedSceneBundle {
        scene:  asset_server.load("scene.glb#Scene0"),
        hook: SceneHook::new_comp(
            move |name: &Name, cmds| hook(&decks, name.as_str(), cmds),
        }),
        ..default()
    });
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(HookPlugin)
            .add_startup_system(load_scene);
        // Use `.with_run_criteria(is_scene_hooked::<Graveyard>)`
        // to only run a system after at least one scene is loaded.
    }
}
```

- [bevy game engine]: https://bevyengine.org/
- [crates.io]: https://crates.io/crates/bevy-scene-hook
- [warlock-source]: https://github.com/team-plover/warlocks-gambit

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

### Version matrix

| bevy | latest supporting version      |
|------|--------|
| 0.8  | 4.0.0 |
| 0.7  | 3.1.0 |
| 0.6  | 1.2.0 |


## License

This library is licensed under Apache 2.0.
