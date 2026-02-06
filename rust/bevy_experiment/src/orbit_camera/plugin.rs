use bevy::{
    app::prelude::*,
    camera::{Camera, ClearColorConfig},
    color::Color,
    ecs::prelude::*,
    prelude::{default, Camera3d, Visibility},
    transform::components::Transform,
};

use crate::orbit_camera::config;
use crate::orbit_camera::controller;
use crate::orbit_camera::events;
use crate::orbit_camera::state;

#[derive(Default)]
pub struct OrbitCameraPlugin;

/// Marker component for the camera pivot entity (parent of the actual camera).
#[derive(Component, Default)]
pub struct CameraPivot;

/// Spawn the camera pivot (parent) with the actual camera as a child entity.
pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            CameraPivot,
            state::OrbitCameraState::default(),
            config::OrbitCameraConfig::default(),
            Transform::default(),
            Visibility::default(),
        ))
        .with_child((
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb_u8(80, 87, 105)),
                ..default()
            },
            Transform::default(),
        ));
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(PreUpdate, events::step)
            .add_systems(Update, controller::step)
            .add_message::<events::OrbitCameraInputEvent>();
    }
}
