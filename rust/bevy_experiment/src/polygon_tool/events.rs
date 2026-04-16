use bevy::{ecs::prelude::*, math::prelude::*};

use crate::common::mouse_interaction::{MouseInteractionPhase, MouseInteractionState};

#[derive(Message)]
pub struct PolygonToolInputEvent {
    /// Screen-space position of a left click (add a control point).
    pub left_click: Option<Vec2>,
    /// A right click occurred (remove the last control point).
    pub right_click: bool,
}

pub fn step(
    mouse_state: Res<MouseInteractionState>,
    mut events: MessageWriter<PolygonToolInputEvent>,
) {
    let left_click = match mouse_state.left.phase {
        MouseInteractionPhase::Clicked(pos) => Some(pos),
        _ => None,
    };
    let right_click = matches!(mouse_state.right.phase, MouseInteractionPhase::Clicked(_));

    events.write(PolygonToolInputEvent {
        left_click,
        right_click,
    });
}
