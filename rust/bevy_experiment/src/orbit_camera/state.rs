// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{ecs::prelude::*, math::f64::*, math::prelude::*};

use crate::common::wgs84_llh::Wgs84Llh;

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
    pub center: Wgs84Llh,
    pub radius: f32,
    // pub upside_down: bool,
    pub elevation: f64,
    pub heading: f64,

    pub center_target: Vec3,
    pub pan_offset_world_space: Vec3,
    pub pan_offset_target: Vec3,
    pub pan_offset_start: Vec3,
    pub drag_start_point: Vec3,
    pub drag_start_lat_lon: Vec3, // TODO: use lat lon!
    pub right_click_start: Vec3,
    //    pub cam:  Camera,
    pub zoom_level_target: f32,
    pub current_zoom_level: f32,
    pub current_euler_angles: Vec3,
    pub euler_angles_target_delta: Vec3,
    pub pan_cursor_position: Vec2,
    pub is_panning: bool,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            center: Wgs84Llh::default(),
            radius: 20.0,
            // upside_down: false,
            elevation: 45.0f64.to_radians(),
            heading: 0.0f64.to_radians(),
            center_target: Vec3::ZERO,
            pan_offset_world_space: Vec3::ZERO,
            pan_offset_target: Vec3::ZERO,
            pan_offset_start: Vec3::ZERO,
            drag_start_point: Vec3::ZERO,
            drag_start_lat_lon: Vec3::ZERO,
            right_click_start: Vec3::ZERO,
            zoom_level_target: 0.0,
            current_zoom_level: 0.0,
            current_euler_angles: Vec3::new(45.0, 0.0, 0.0),
            euler_angles_target_delta: Vec3::ZERO,
            pan_cursor_position: Vec2::ZERO,
            is_panning: false,
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
