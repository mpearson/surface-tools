// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{ecs::prelude::*, math::prelude::*};

// // Bundle to spawn our custom camera easily
// #[derive(Bundle, Default)]
// pub struct PanOrbitCameraBundle {
//     pub camera: Camera3dBundle,
//     pub state: PanOrbitState,
//     pub settings: Config,
// }

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct OrbitCameraState {
    pub center: Vec3,
    pub radius: f32,
    // pub upside_down: bool,
    pub elevation: f32,
    pub heading: f32,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 1.0,
            // upside_down: false,
            elevation: 0.0,
            heading: 0.0,
        }
    }
}

// /// A 3rd person camera that orbits around the target.
// #[derive(Clone, Component, Copy, Debug, Reflect)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
// #[reflect(Component, Default, Debug)]
// pub struct OrbitCameraController {
//     pub enabled: bool,
//     pub mouse_rotate_sensitivity: Vec2,
//     pub mouse_translate_sensitivity: Vec2,
//     pub mouse_wheel_zoom_sensitivity: f32,
//     pub pixels_per_line: f32,
//     pub smoothing_weight: f32,
// }

// impl Default for OrbitCameraController {
//     fn default() -> Self {
//         Self {
//             mouse_rotate_sensitivity: Vec2::splat(0.08),
//             mouse_translate_sensitivity: Vec2::splat(0.1),
//             mouse_wheel_zoom_sensitivity: 0.2,
//             smoothing_weight: 0.8,
//             enabled: true,
//             pixels_per_line: 53.0,
//         }
//     }
// }
