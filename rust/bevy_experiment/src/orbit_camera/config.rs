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
// #[derive(Component)]
// pub struct PanOrbitState {
//     pub center: Vec3,
//     pub radius: f32,
//     pub upside_down: bool,
//     pub pitch: f32,
//     pub yaw: f32,
// }

/// The configuration of the pan-orbit controller
#[derive(Component)]
pub struct OrbitCameraConfig {
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub orbit_sensitivity: Vec2,
    pub scroll_wheel_pixels_per_line: f32,
    // pub orbit_sensitivity_x: f32,
    // pub orbit_sensitivity_y: f32,
    pub max_distance: f32,
    pub min_distance: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub min_theta: f32,
    pub max_theta: f32,
    // /// World units per pixel of mouse motion
    // pub pan_sensitivity: f32,
    // /// Radians per pixel of mouse motion
    // pub orbit_sensitivity: f32,
    // /// Exponent per pixel of mouse motion
    // pub zoom_sensitivity: f32,
    // /// Key to hold for panning
    // pub pan_key: Option<KeyCode>,
    // /// Key to hold for orbiting
    // pub orbit_key: Option<KeyCode>,
    // /// Key to hold for zooming
    // pub zoom_key: Option<KeyCode>,
    // /// What action is bound to the scroll wheel?
    // pub scroll_action: Option<PanOrbitAction>,
    // /// For devices with a notched scroll wheel, like desktop mice
    // pub scroll_line_sensitivity: f32,
    // /// For devices with smooth scrolling, like touchpads
    // pub scroll_pixel_sensitivity: f32,
}

impl Default for OrbitCameraConfig {
    fn default() -> Self {
        Self {
            pan_sensitivity: 8.0,
            zoom_sensitivity: 0.2,
            scroll_wheel_pixels_per_line: 16.0,
            // orbit_sensitivity_x: 0.4,
            // orbit_sensitivity_y: 0.3,
            orbit_sensitivity: Vec2::new(0.4, 0.3),
            max_distance: 10000.0,
            min_distance: 50.0,
            min_zoom: 1.0,
            max_zoom: 20.0,
            min_theta: -80.0,
            max_theta: 80.0,
            // pan_sensitivity: 0.001,                 // 1000 pixels per world unit
            // orbit_sensitivity: 0.1f32.to_radians(), // 0.1 degree per pixel
            // zoom_sensitivity: 0.01,
            // pan_key: Some(KeyCode::ControlLeft),
            // orbit_key: Some(KeyCode::AltLeft),
            // zoom_key: Some(KeyCode::ShiftLeft),
            // scroll_action: Some(PanOrbitAction::Zoom),
            // scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            // scroll_pixel_sensitivity: 1.0,
        }
    }
}
