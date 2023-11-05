use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_scene_hook::HookPlugin))
        .add_systems(Startup, load_scene)
        .run();
}

#[derive(Component)]
struct Box;

#[derive(Component)]
struct Ground;

fn load_scene(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(6.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
    cmds.spawn(PointLightBundle {
        transform: Transform::from_xyz(8.0, 8.0, 6.0),
        ..default()
    });

    // Check console output to see which entities are hooked.
    // You can run arbitrary commands on those entities, add components and bundles, etc.
    cmds.spawn(HookedSceneBundle {
        scene: SceneBundle {
            scene: asset_server.load("box.gltf#Scene0"),
            ..default()
        },
        hook: SceneHook::new(|entity, cmds| {
            match entity.get::<Name>().map(|t| t.as_str()) {
                Some("Box") => {
                    cmds.insert(Box);
                    info!("Hooked Box marker")
                }
                Some("Ground") => {
                    cmds.insert(Ground);
                    info!("Hooked Ground marker")
                }
                _ => {}
            };
        }),
    });
}
