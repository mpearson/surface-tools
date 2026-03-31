use bevy::{ecs::prelude::*, math::f64::*, math::prelude::*};

pub struct PanState {
    pub start_screen_space: Vec2,
    pub offset_screen_space: Vec2,
    pub start_world_space: DVec3,
    pub start_radius: f64,
}

pub struct ZoomState {
    pub start_cursor_screen_space: Vec2,
    pub start_world_space: DVec3,
    pub start_radius: f64,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct OrbitCameraState {
    pub radius: f64,

    /// f64 precision rotation of the NED frame origin (source of truth for pan).
    /// The pivot entity's Transform.rotation is downcast from this each frame.
    pub center_rotation: DQuat,

    /// Point on the earth surface the camera is centered on.
    /// Derived each frame from camera_center_rotation.
    pub center_position_world_space: DVec3,

    pub center_rotation_ref: DQuat,
    pub pan: Option<PanState>,

    pub zoom_rotation_reference: DQuat,
    pub zoom: Option<ZoomState>,

    pub zoom_level_ref: f64,
    pub current_zoom_level: f64,
    pub current_euler_angles: Vec3,
    pub euler_angles_target_delta: Vec3,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            radius: 20.0,
            center_rotation: DQuat::IDENTITY,
            center_position_world_space: DVec3::new(0.0, 0.0, 1.0),
            center_rotation_ref: DQuat::IDENTITY,
            pan: None,
            zoom_rotation_reference: DQuat::IDENTITY,
            zoom: None,
            zoom_level_ref: 0.0,
            current_zoom_level: 0.0,
            current_euler_angles: Vec3::new(0.0, 0.0, 0.0),
            euler_angles_target_delta: Vec3::ZERO,
        }
    }
}
