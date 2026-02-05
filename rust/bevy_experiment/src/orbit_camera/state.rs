// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{ecs::prelude::*, math::f64::*, math::prelude::*, prelude::GlobalTransform};

use crate::common::wgs84_llh::Wgs84Llh;

// // Bundle to spawn our custom camera easily
// #[derive(Bundle, Default)]
// pub struct PanOrbitCameraBundle {
//     pub camera: Camera3dBundle,
//     pub state: PanOrbitState,
//     pub settings: Config,
// }

pub struct PanState {
    pub start_screen_space: Vec2,
    pub offset_screen_space: Vec2,
    pub start_world_space: DVec3,
    pub start_radius: f64,
    pub current_world_space: DVec3,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct OrbitCameraState {
    // pub center: Wgs84Llh,
    pub radius: f64,
    // pub upside_down: bool,
    // pub elevation: f64,
    // pub heading: f64,
    pub pan_rotation_target: DQuat,
    pub pan: Option<PanState>,
    // pub pan_start_screen_space: Vec2,
    // pub pan_offset_screen_space: Vec2,
    // pub pan_start_world_space: DVec2,

    // pub pan_offset_world_space: DVec2,
    // pub pan_offset_target: Vec3,
    // pub pan_offset_start: Vec3,
    // pub drag_start_point: Vec3,
    // pub drag_start_lat_lon: Vec3, // TODO: use lat lon!
    pub right_click_start: Vec3,
    pub zoom_level_target: f64,
    pub current_zoom_level: f64,
    pub current_euler_angles: Vec3,
    pub euler_angles_target_delta: Vec3,
    // pub pan_cursor_position: Vec2,
    // pub is_panning: bool,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            // center: Wgs84Llh::default(),
            radius: 20.0,
            // elevation: 45.0f64.to_radians(),
            // heading: 0.0f64.to_radians(),
            pan_rotation_target: DQuat::IDENTITY,
            pan: None,
            right_click_start: Vec3::ZERO,
            zoom_level_target: 0.0,
            current_zoom_level: 0.0,
            current_euler_angles: Vec3::new(45.0, 0.0, 0.0),
            euler_angles_target_delta: Vec3::ZERO,
            // pan_cursor_position: Vec2::ZERO,
            // is_panning: false,
        }
    }
}
