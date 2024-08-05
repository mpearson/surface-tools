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
};

use super::config::OrbitCameraConfig;

#[derive(Event)]
pub enum OrbitCameraInput {
    Orbit(Vec2),
    TranslateTarget(Vec2),
    Zoom(f32),
}

pub fn default_input_map(
    mut events: EventWriter<OrbitCameraInput>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controllers: Query<&OrbitCameraConfig>,
) {
    // Can only control one camera at a time.
    // let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
    let controller = if let Some(controller) = controllers.iter().next() {
        controller
    } else {
        return;
    };
    let OrbitCameraConfig {
        pan_sensitivity,
        zoom_sensitivity,
        orbit_sensitivity,
        scroll_wheel_pixels_per_line,
        // orbit_sensitivity_x,
        // orbit_sensitivity_y,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    if keyboard.pressed(KeyCode::ControlLeft) {
        events.send(OrbitCameraInput::Orbit(orbit_sensitivity * cursor_delta));
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(OrbitCameraInput::TranslateTarget(
            pan_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            // scale the event magnitude per pixel or per line
            MouseScrollUnit::Pixel => event.y / scroll_wheel_pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * zoom_sensitivity;
    }
    events.send(OrbitCameraInput::Zoom(scalar));
}
