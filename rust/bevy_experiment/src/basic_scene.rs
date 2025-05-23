use bevy::prelude::*;
use std::f32::consts::TAU;

// #[derive(Component)]
// pub struct CubeRotator {
//     frequency: f32,
// }

// impl Default for CubeRotator {
//     fn default() -> Self {
//         Self { frequency: 1.0 }
//     }
// }

/// set up a simple 3D scene
pub fn spawn_cube(
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
}

// pub fn rotate_cube(time: Res<Time>, mut query: Query<(&mut Transform, &CubeRotator)>) {
//     for (mut transform, cube) in &mut query {
//         transform.rotate(Quat::from_rotation_y(
//             time.delta_secs() * cube.frequency * TAU,
//         ));
//     }
// }

pub struct BasicScenePlugin;

impl Plugin for BasicScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_cube,));
        // .add_systems(Update, (basic_scene::rotate_cube,));
        // .add_systems(Startup, test_system::print_map_tiles)
        // .add_systems(Update, test_system::print_map_tiles);
    }
}
