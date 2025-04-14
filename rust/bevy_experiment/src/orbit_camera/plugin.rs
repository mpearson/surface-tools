// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    app::prelude::*,
    color::Color,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    prelude::{default, Camera3dBundle, ReflectDefault},
    reflect::Reflect,
    render::camera::{Camera, ClearColorConfig},
    time::Time,
    transform::components::Transform,
};

use crate::orbit_camera::config;
use crate::orbit_camera::controller;
use crate::orbit_camera::events;
use crate::orbit_camera::state;

#[derive(Default)]
pub struct OrbitCameraPlugin;
//  {
// pub override_input_system: bool,
// }

// impl OrbitCameraPlugin {
//     pub fn new(override_input_system: bool) -> Self {
//         Self {
//             override_input_system,
//         }
//     }
// }

// Bundle to spawn our custom camera easily
#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3dBundle,
    pub state: state::OrbitCameraState,
    pub config: config::OrbitCameraConfig,
}

/// create the actual camera object
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(PanOrbitCameraBundle {
        camera: Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb_u8(80, 87, 105)),
                ..default()
            },
            ..default()
        },
        state: state::OrbitCameraState {
            // center: Vec3::new(1.0, 2.0, 3.0),
            radius: 20.0,
            elevation: 90.0f64.to_radians(),
            heading: 0.0f64.to_radians(),
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

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        // let app = app
        app
            // .add_systems(PreUpdate, on_controller_enabled_changed)
            .add_systems(Startup, spawn_camera)
            .add_systems(PreUpdate, events::default_input_map)
            .add_systems(Update, controller::update)
            .add_event::<events::OrbitCameraInput>();

        // if !self.override_input_system {
        //     app.add_systems(Update, default_input_map);
        // }
    }
}
