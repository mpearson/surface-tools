use std::f32::consts::TAU;
// use bevy::ecs::schedule::IntoSystemConfigs;
// use std::env;
use bevy::prelude::*;

use crate::camera::{PanOrbitCameraBundle, PanOrbitState};
// use bevy::prelude::Startup;
// use bevy::prelude::Update;
// use crate::camera::{PanOrbitCameraBundle, PanOrbitState};

#[derive(Component)]
pub struct CubeRotator {
    frequency: f32,
}

impl Default for CubeRotator {
    fn default() -> Self {
        Self { frequency: 1.0 }
    }
}

/// set up a simple 3D scene
pub fn spawn_spinning_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // old camera
    // spawn_camera(commands);
    // commands.spawn(PanOrbitCameraBundle {
    //     camera: Camera3dBundle {
    //         camera: Camera {
    //             clear_color: ClearColorConfig::Custom(Color::srgb_u8(80, 87, 105)),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     state: PanOrbitState {
    //         // center: Vec3::new(1.0, 2.0, 3.0),
    //         radius: 20.0,
    //         pitch: 90.0f32.to_radians(),
    //         yaw: 0.0f32.to_radians(),
    //         ..default()
    //     },
    //     // transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     // transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}

pub fn rotate_cube(time: Res<Time>, mut query: Query<(&mut Transform, &CubeRotator)>) {
    for (mut transform, cube) in &mut query {
        transform.rotate(Quat::from_rotation_y(
            time.delta_secs() * cube.frequency * TAU,
        ));
    }
}
