use std::f32::consts::TAU;
// use bevy::ecs::schedule::IntoSystemConfigs;
// use std::env;
use bevy::prelude::*;
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        CubeRotator { frequency: 0.5 },
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

pub fn rotate_cube(time: Res<Time>, mut query: Query<(&mut Transform, &CubeRotator)>) {
    for (mut transform, cube) in &mut query {
        transform.rotate(Quat::from_rotation_y(
            time.delta_seconds() * cube.frequency * TAU,
        ));
    }
}
