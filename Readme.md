# Bevy Scene hook

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Latest version](https://img.shields.io/crates/v/bevy_scene_hook.svg)](https://crates.io/crates/bevy_scene_hook)
[![Apache 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](./LICENSE)
[![Documentation](https://docs.rs/bevy-scene-hook/badge.svg)](https://docs.rs/bevy-scene-hook/)

A one-file proof of concept for adding components ad-hoc within code to
entities spawned through scenes (such as `gltf` files) in [the bevy game
engine](https://bevyengine.org/).

If you don't mind adding such a small dependency to your code rather than
copy/pasting the code as a module, you can get it from [crates.io](https://crates.io/crates/bevy-scene-hook).

## Usage

The following snippet of code is extracted from [Warlock's Gambit source
code](https://github.com/team-plover/warlocks-gambit).

It loads the `scene.glb` file when the game state becomes `GameState::Loading`.
It then runs the `Scene::hook` method once using the `Scene::when_not_spawned` run
criteria, `Scene::hook` runs the `Scene::hook_named_node` method for each named
entity in the scene and adds specific components to them. Most notably
the animation ones. It then leaves the `GameState::Loading` state using the
`Scene::when_spawned` run criteria.

It is possible to name object in `glb` scenes in blender using the Outliner
dock (the tree view at the top right) and double-clicking object names.

```rust
use bevy::{
    ecs::system::EntityCommands,
    prelude::{Plugin as BevyPlugin, *},
};
use bevy_scene_hook::{SceneHook, SceneInstance};
use crate::{
    animate::Animated,
    camera::PlayerCam,
    common_components::{OppoCardSpawner, OppoHand, PlayerCardSpawner, PlayerHand, PlayerSleeve},
    pile::{Pile, PileType},
    state::GameState,
};
pub enum Scene {}
impl SceneHook for Scene {
    fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {
        match name.as_str() {
            "PlayerPerspective_Orientation" => cmds.insert(PlayerCam),
            "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
            "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
            "OppoHand" => cmds.insert_bundle((OppoHand, Animated::bob(1.0, 0.3, 6.0))),
            "PlayerHand" => cmds.insert_bundle((PlayerHand, Animated::bob(2.0, 0.05, 7.0))),
            "Pile" => cmds.insert(Pile::new(PileType::War)),
            "OppoPile" => cmds.insert(Pile::new(PileType::Oppo)),
            "PlayerPile" => cmds.insert(Pile::new(PileType::Player)),
            "ManBody" => cmds.insert(Animated::breath(0.0, 0.03, 6.0)),
            "ManHead" => cmds.insert(Animated::bob(6. / 4., 0.1, 6.0)),
            "Bird" => cmds.insert(Animated::breath(0.0, 0.075, 5.0)),
            "BirdEyePupilla" => cmds.insert(Animated::bob(5. / 4., 0.02, 5.0)),
            "PlayerSleeveStash" => cmds.insert(PlayerSleeve),
            _ => cmds,
        };
    }
}
fn setup_scene(
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
) {
    let gltf = scene_spawner.spawn(asset_server.load("scene.glb#Scene0"));
    cmds.spawn().insert(SceneInstance::<Scene>::new(gltf));
}
fn exit_load_state(mut state: ResMut<State<GameState>>) {
    if state.current() == &GameState::LoadScene {
        state.set(GameState::Playing).unwrap();
    }
}
pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Loading).with_system(setup_scene))
            .add_system(exit_load_state.with_run_criteria(Scene::when_spawned))
            .add_system(Scene::hook.with_run_criteria(Scene::when_not_spawned));
    }
}
```

## Limitations

* You will need to keep track of what you name what.
* The bevy gltf loader currently doesn't name entities associated with a mesh (the entity
  with the `Handle<Mesh>` component), this is [fixed in a pending PR](https://github.com/bevyengine/bevy/pull/4119).

## License

This library is licensed under Apache 2.0.
