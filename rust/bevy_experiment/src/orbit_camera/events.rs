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
pub struct OrbitCameraInput {
    pub pan_delta: Vec2,
    pub orbit_delta: Vec2,
    pub zoom_delta: f32,
}

pub fn default_input_map(
    mut events: EventWriter<OrbitCameraInput>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
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
        // orbit_sensitivity_x,
        // orbit_sensitivity_y,
        ..
    } = *config;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    // if keyboard.pressed(KeyCode::ControlLeft) {
    //     events.send(OrbitCameraInput::Orbit(orbit_sensitivity * cursor_delta));
    // }

    // if mouse_buttons.pressed(MouseButton::Right) {
    //     events.send(OrbitCameraInput::TranslateTarget(
    //         pan_sensitivity * cursor_delta,
    //     ));
    // }

    let pan_delta = Vec2::ZERO;
    let orbit_delta = Vec2::ZERO;

    let mut zoom_delta = 0.0;
    for event in mouse_wheel_reader.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            // scale the event magnitude per pixel or per line
            MouseScrollUnit::Pixel => event.y / scroll_wheel_pixels_per_line,
        };
        zoom_delta -= scroll_amount * zoom_sensitivity;
    }
    // events.send(OrbitCameraInput::Zoom(zoom_delta));
    events.send(OrbitCameraInput {
        pan_delta,
        orbit_delta,
        zoom_delta,
    });
}
