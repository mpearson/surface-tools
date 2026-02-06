use bevy::{ecs::prelude::*, math::f64::*, math::prelude::*};

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
    pub radius: f64,

    /// f64-precision rotation of the camera pivot (source of truth for pan).
    /// The pivot entity's Transform.rotation is derived from this each frame.
    pub camera_center_rotation: DQuat,

    /// Point on the earth surface the camera is centered on.
    /// Derived each frame from camera_center_rotation.
    pub camera_center_world_space: DVec3,

    pub pan_rotation_target: DQuat,
    pub pan: Option<PanState>,

    pub right_click_start: Vec3,
    pub zoom_level_target: f64,
    pub current_zoom_level: f64,
    pub current_euler_angles: Vec3,
    pub euler_angles_target_delta: Vec3,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            radius: 20.0,
            camera_center_rotation: DQuat::IDENTITY,
            camera_center_world_space: DVec3::new(0.0, 0.0, 1.0),
            pan_rotation_target: DQuat::IDENTITY,
            pan: None,
            right_click_start: Vec3::ZERO,
            zoom_level_target: 0.0,
            current_zoom_level: 0.0,
            current_euler_angles: Vec3::new(45.0, 0.0, 0.0),
            euler_angles_target_delta: Vec3::ZERO,
        }
    }
}
