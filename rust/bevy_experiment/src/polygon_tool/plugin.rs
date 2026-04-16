use bevy::{app::prelude::*, ecs::prelude::*};

use crate::common::mouse_interaction::classify_mouse_interaction;

use super::config::PolygonToolConfig;
use super::controller;
use super::events;
use super::state::PolygonToolState;

fn spawn_polygon_tool(mut commands: Commands) {
    commands.spawn((PolygonToolState::default(), PolygonToolConfig::default()));
}

pub struct PolygonToolPlugin;

impl Plugin for PolygonToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_polygon_tool)
            .add_systems(
                PreUpdate,
                events::step.after(classify_mouse_interaction),
            )
            .add_systems(Update, controller::step)
            .add_message::<events::PolygonToolInputEvent>();
    }
}
