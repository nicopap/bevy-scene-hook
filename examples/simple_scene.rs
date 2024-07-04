//! This demonstrates the standard and reload hooks, and how to use them
//! to add components to a `gltf` file.
//!
//! Note that `sample-scene.gltf` is a cube with 4 "empty" children, each with
//! a color name.
use std::f32::consts::TAU;

use bevy::{color::palettes::css, prelude::*};
use bevy_scene_hook::{reload, HookPlugin, HookedSceneBundle, SceneHook};

// You can open this file in Blender and modify it, or just open it with a
// text editor and fiddle with it.
const SAMPLE: &str = "sample-scene.gltf#Scene0";

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HookPlugin, reload::Plugin))
        .add_systems(Startup, (setup, load_scenes))
        .add_systems(Update, (reload_scene, show_gizmos, rotate_cube))
        .run();
}

#[derive(Clone, Copy)]
enum Shape {
    Sphere,
    Cube,
    Cone,
    Cylinder,
}

#[derive(Component)]
struct ShowGizmo {
    color: Color,
    shape: Shape,
}

fn show_gizmos(mut gizmos: Gizmos, to_show: Query<(&GlobalTransform, &ShowGizmo)>) {
    for (trans, to_show) in &to_show {
        let (_, rot, pos) = trans.to_scale_rotation_translation();
        let cuboid = Transform {
            scale: Vec3::splat(0.2),
            translation: pos,
            rotation: rot,
        };
        let cone = Cone { radius: 0.1, height: 0.3 };
        let cylinder = Cylinder { radius: 0.1, half_height: 0.15 };
        match to_show.shape {
            Shape::Sphere => {
                gizmos.sphere(pos, rot, 0.2, to_show.color);
            }
            Shape::Cube => {
                gizmos.cuboid(cuboid, to_show.color);
            }
            Shape::Cone => {
                gizmos.primitive_3d(&cone, pos, rot, to_show.color);
            }
            Shape::Cylinder => {
                gizmos.primitive_3d(&cylinder, pos, rot, to_show.color);
            }
        }
    }
}

#[derive(Component)]
struct Cube(f32);

fn rotate_cube(mut cubes: Query<(&Cube, &mut Transform)>) {
    for (speed, mut trans) in &mut cubes {
        trans.rotation = (trans.rotation * Quat::from_rotation_z(speed.0)).normalize();
    }
}

fn setup(mut cmds: Commands, mut gizmo_conf: ResMut<GizmoConfigStore>) {
    let config = gizmo_conf.config_mut::<DefaultGizmoConfigGroup>().0;
    config.depth_bias = 0.;

    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5., 0., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // example instructions
    let instr = "1: reload\n2: change reloadable scene\n3: delete reloadable scene (permanent)";
    cmds.spawn(TextBundle::from_section(instr, default()));
}

fn load_scenes(mut cmds: Commands, server: Res<AssetServer>) {
    let show_gizmo = |color, shape| ShowGizmo { color, shape };
    // ## HookedSceneBundle, standard usage ##

    cmds.spawn(HookedSceneBundle {
        scene: SceneBundle {
            scene: server.load(SAMPLE),
            transform: Transform::from_xyz(0., 0., -2.),
            ..default()
        },
        hook: SceneHook::new(move |entity, cmds| {
            // You are not limited to matching the `Name`, you could also
            // parse it and add different thing based on the name. For example,
            // you could convert the name into a color instead of hardcoding the color.
            match entity.get().map(Name::as_str) {
                Some("yellow") => cmds.insert(show_gizmo(css::YELLOW.into(), Shape::Sphere)),
                Some("red") => cmds.insert(show_gizmo(css::RED.into(), Shape::Cone)),
                Some("green") => cmds.insert(show_gizmo(css::GREEN.into(), Shape::Cone)),
                Some("blue") => cmds.insert(show_gizmo(css::BLUE.into(), Shape::Sphere)),
                Some("Cube") => cmds.insert(Cube(-0.025)),
                _ => cmds,
            };
        }),
    });

    // ## reload::SceneBundle, advanced usage ##

    cmds.spawn(reload::SceneBundle {
        scene: SceneBundle {
            scene: server.load(SAMPLE),
            transform: Transform::from_xyz(0., 0., 2.),
            ..default()
        },
        reload: reload::Hook::new(move |entity, cmds, _world, _root| {
            match entity.get().map(Name::as_str) {
                Some("yellow") => cmds.insert(show_gizmo(css::YELLOW.into(), Shape::Cube)),
                Some("red") => cmds.insert(show_gizmo(css::RED.into(), Shape::Cylinder)),
                Some("green") => cmds.insert(show_gizmo(css::GREEN.into(), Shape::Cylinder)),
                Some("blue") => cmds.insert(show_gizmo(css::BLUE.into(), Shape::Cube)),
                Some("Cube") => cmds.insert(Cube(0.05)),
                _ => cmds,
            };
        }),
    });
}

fn reload_scene(
    keys: Res<ButtonInput<KeyCode>>,
    mut reload_scene: Query<(Entity, &mut reload::Hook)>,
    children: Query<&Children>,
    mut trans: Query<(&mut ShowGizmo, &mut Transform)>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        for (_, mut hook) in &mut reload_scene {
            hook.state = reload::State::MustReload;
        }
    }
    if keys.just_pressed(KeyCode::Digit2) {
        for (entity, _) in &reload_scene {
            let descendants = children.iter_descendants(entity);
            let mut iter = trans.iter_many_mut(descendants);
            while let Some((mut giz, mut trans)) = iter.fetch_next() {
                let color = giz.color.to_srgba().to_vec3();
                giz.color = Color::hsl((color.x + 10.) % 360.0, color.y, color.z);

                trans.rotation *= Quat::from_rotation_x(TAU / 11.);
            }
        }
    }
    if keys.just_pressed(KeyCode::Digit3) {
        for (_, mut hook) in &mut reload_scene {
            hook.state = reload::State::MustDelete;
        }
    }
}
