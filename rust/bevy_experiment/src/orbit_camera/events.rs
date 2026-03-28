// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::{
    ecs::prelude::*,
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    // math::f64::*,
    math::prelude::*,
    window::{CursorMoved, PrimaryWindow, Window},
};

use super::config::OrbitCameraConfig;

/// Abstracted input event for orbit camera control.
#[derive(Message)]
pub struct OrbitCameraInputEvent {
    pub pan_started: bool,
    // pub pan_start_screen_space: Option<Vec2>,
    /// Raw mouse motion delta while panning (from MouseMotion, works outside window).
    pub pan_delta: Option<Vec2>,
    /// Absolute cursor position (populated every frame when available).
    /// When panning, the controller should prefer this over accumulating pan_delta.
    pub cursor_position: Option<Vec2>,
    pub orbit_delta: Option<Vec2>,
    pub zoom_delta: f64,
}

/// Mouse input mapping system.
/// Maps from raw mouse and keyboard events to orbit camera input events.
/// This mapping may change over time (e.g. panning may be changed to middle mouse button instead of
/// left), but [OrbitCameraInputEvent] will remain the same.
pub fn step(
    mut events: MessageWriter<OrbitCameraInputEvent>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut last_cursor_position: Local<Option<Vec2>>,
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
        // pan_sensitivity: _,
        zoom_sensitivity,
        orbit_sensitivity,
        scroll_wheel_pixels_per_line,
        ..
    } = *config;

    // Track the cursor position from CursorMoved events, since window.cursor_position()
    // may not be populated on all platforms (e.g. native builds without a prior mouse move).
    for event in cursor_moved_events.read() {
        *last_cursor_position = Some(event.position);
    }
    let cursor_position = last_cursor_position.or(window.cursor_position());

    // There may be multiple mouse move events per frame, so we need to accumulate the deltas.
    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    let pan_started = mouse_buttons.just_pressed(MouseButton::Left) && cursor_position.is_some();

    // Depending on which mouse button is pressed, the mouse delta is applied to pan and/or orbit.
    let is_panning = mouse_buttons.pressed(MouseButton::Left);
    let pan_delta = is_panning.then_some(cursor_delta);
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
        zoom_delta -= scroll_amount as f64 * zoom_sensitivity;
    }

    events.write(OrbitCameraInputEvent {
        pan_started,
        pan_delta,
        cursor_position,
        orbit_delta,
        zoom_delta,
    });
}
