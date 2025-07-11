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

#[derive(Event)]
pub struct OrbitCameraInput {
    pub pan_start: Option<Vec2>,
    pub pan_delta: Vec2,
    pub orbit_delta: Vec2,
    pub zoom_delta: f32,
}

pub fn default_input_map(
    mut events: EventWriter<OrbitCameraInput>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
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
        pan_sensitivity,
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
        window.cursor_position()
    } else {
        None
    };

    // Depending on which mouse button is pressed, the mouse delta is applied to pan and/or orbit.
    let mut pan_delta = Vec2::ZERO;
    let mut orbit_delta = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Left) {
        pan_delta = cursor_delta * pan_sensitivity;
    }
    if mouse_buttons.pressed(MouseButton::Right) {
        orbit_delta = cursor_delta * orbit_sensitivity;
    }

    let mut zoom_delta = 0.0;
    for event in mouse_wheel_reader.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            // scale the event magnitude per pixel or per line
            MouseScrollUnit::Pixel => event.y / scroll_wheel_pixels_per_line,
        };
        zoom_delta -= scroll_amount * zoom_sensitivity;
    }
    events.send(OrbitCameraInput {
        pan_start,
        pan_delta,
        orbit_delta,
        zoom_delta,
    });
}
