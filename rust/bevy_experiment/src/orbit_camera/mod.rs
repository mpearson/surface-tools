// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    prelude::Camera3dBundle,
    prelude::ReflectDefault,
    reflect::Reflect,
    time::Time,
    transform::components::Transform,
};

pub mod config;
// use config::*;

pub mod state;
// use state::*;

mod events;
// use event::*;

mod controller;
// use controller::*;

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
    pub settings: config::OrbitCameraConfig,
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .add_systems(PreUpdate, on_controller_enabled_changed)
            .add_systems(Update, controller::control_system)
            .add_event::<events::OrbitCameraInput>();

        if !self.override_input_system {
            app.add_systems(Update, default_input_map);
        }
    }
}
