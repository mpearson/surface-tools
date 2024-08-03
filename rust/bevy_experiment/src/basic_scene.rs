// use bevy::ecs::schedule::IntoSystemConfigs;
// use std::env;
use bevy::prelude::*;
// use bevy::prelude::Startup;
// use bevy::prelude::Update;
use crate::camera::{PanOrbitCameraBundle, PanOrbitState};

/// set up a simple 3D scene
pub fn spawn_stuff(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 1.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    // spawn_camera(commands);

    commands.spawn(PanOrbitCameraBundle {
        camera: Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb_u8(80, 87, 105)),
                ..default()
            },
            ..default()
        },
        state: PanOrbitState {
            // center: Vec3::new(1.0, 2.0, 3.0),
            radius: 20.0,
            pitch: 45.0f32.to_radians(),
            yaw: 0.0f32.to_radians(),
            ..default()
        },
        // transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        // transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}
