// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{ecs::prelude::*, math::f64::*, math::prelude::*};

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
    pub center: DVec3,
    pub radius: f64,
    // pub upside_down: bool,
    pub elevation: f64,
    pub heading: f64,

    pub center_target: DVec3,
    pub pan_offset_world_space: DVec3,
    pub pan_offset_target: DVec3,
    pub drag_start_point: DVec3,
    pub drag_start_lat_lon: DVec3, // TODO: use lat lon!
    pub right_click_start: DVec3,
    //    pub cam:  Camera,
    pub zoom_level_target: f64,
    pub current_zoom_level: f64,
    pub current_euler_angles: DVec3,
    pub euler_angles_target_delta: DVec3,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            radius: 1.0,
            // upside_down: false,
            elevation: 0.0,
            heading: 0.0,
            center_target: DVec3::ZERO,
            pan_offset_world_space: DVec3::ZERO,
            pan_offset_target: DVec3::ZERO,
            drag_start_point: DVec3::ZERO,
            drag_start_lat_lon: DVec3::ZERO,
            right_click_start: DVec3::ZERO,
            zoom_level_target: 0.0,
            current_zoom_level: 0.0,
            current_euler_angles: DVec3::ZERO,
            euler_angles_target_delta: DVec3::ZERO,
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
