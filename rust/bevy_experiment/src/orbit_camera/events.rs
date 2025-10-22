// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    ecs::prelude::*,
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    window::{PrimaryWindow, Window},
};

use super::config::OrbitCameraConfig;

/// Abstracted input event for orbit camera control.
#[derive(Message)]
pub struct OrbitCameraInputEvent {
    pub pan_start: Option<Vec2>,
    pub pan_delta: Option<Vec2>,
    pub orbit_delta: Option<Vec2>,
    pub zoom_delta: f32,
}

/// Mouse input mapping system.
/// Maps from raw mouse and keyboard events to orbit camera input events.
/// This mapping may change over time (e.g. panning may be changed to middle mouse button instead of
/// left), but [OrbitCameraInputEvent] will remain the same.
pub fn step(
    mut events: MessageWriter<OrbitCameraInputEvent>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // keyboard: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    configs: Query<&OrbitCameraConfig>,
) {
    // Can only control one camera at a time.
    // let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
    let config = if let Some(config) = configs.iter().next() {
        config
    } else {
        return;
    };
    let OrbitCameraConfig {
        pan_sensitivity: _,
        zoom_sensitivity,
        orbit_sensitivity,
        scroll_wheel_pixels_per_line,
        ..
    } = *config;

    // There may be multiple mouse move events per frame, so we need to accumulate the deltas.
    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    // If the left mouse button was pressed during this frame, get the current mouse position
    // since this will be used to determine the "grab point" on the Earth's surface.
    let pan_start = if mouse_buttons.just_pressed(MouseButton::Left) {
        // TODO: figure out which object has been grabbed. Is it the surface of the earth? 2D or
        // 3D terrain? Or a waypoint or something?
        // For now, we should assume it's the surface of a smooth spherical earth.

        // Whatever we intersect, we need to get a point in spherical coordinates (lat, lon, alt?)
        // which becomes a "handle" with which to rotate the ellipsoid. On subsequent frames, we
        // must then compute the lat/lon deltas needed to move that handle point onto the new screen
        // ray passing through the mouse position.

        window.cursor_position()
    } else {
        None
    };

    // Depending on which mouse button is pressed, the mouse delta is applied to pan and/or orbit.
    let pan_delta = mouse_buttons
        .pressed(MouseButton::Left)
        .then_some(cursor_delta);
    let orbit_delta = mouse_buttons
        .pressed(MouseButton::Right)
        .then_some(cursor_delta * -orbit_sensitivity);

    let mut zoom_delta = 0.0;
    for event in mouse_wheel_reader.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            // scale the event magnitude per pixel or per line
            MouseScrollUnit::Pixel => event.y / scroll_wheel_pixels_per_line,
        };
        zoom_delta -= scroll_amount * zoom_sensitivity;
    }
    events.write(OrbitCameraInputEvent {
        pan_start,
        pan_delta,
        orbit_delta,
        zoom_delta,
    });
}
