use bevy::{
    app::prelude::*,
    ecs::prelude::*,
    input::prelude::*,
    math::prelude::*,
    window::{PrimaryWindow, Window},
};

const CLICK_DRAG_THRESHOLD_PX: f32 = 3.0;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum MouseInteractionPhase {
    #[default]
    Idle,
    /// Button is down but cursor hasn't moved beyond the dead zone yet.
    PendingClassification,
    /// Cursor moved >= threshold pixels from press position. This is a drag.
    Dragging,
    /// Button released within threshold. This is a click.
    /// Contains the screen-space position of the click (the original press position).
    /// Only active for one frame (the frame of mouse release), then resets to Idle.
    Clicked(Vec2),
}

#[derive(Default)]
pub struct ButtonInteractionState {
    pub press_position: Option<Vec2>,
    pub phase: MouseInteractionPhase,
}

#[derive(Resource, Default)]
pub struct MouseInteractionState {
    pub left: ButtonInteractionState,
    pub right: ButtonInteractionState,
}

fn classify_button(
    state: &mut ButtonInteractionState,
    just_pressed: bool,
    pressed: bool,
    just_released: bool,
    cursor_position: Option<Vec2>,
) {
    // Clear single-frame Clicked state from previous frame.
    if matches!(state.phase, MouseInteractionPhase::Clicked(_)) {
        state.phase = MouseInteractionPhase::Idle;
        state.press_position = None;
    }

    if just_pressed {
        state.press_position = cursor_position;
        state.phase = if cursor_position.is_some() {
            MouseInteractionPhase::PendingClassification
        } else {
            MouseInteractionPhase::Idle
        };
    }

    // Check if cursor has moved beyond the dead zone while pending.
    if state.phase == MouseInteractionPhase::PendingClassification {
        if let (Some(press_pos), Some(cursor_pos)) = (state.press_position, cursor_position) {
            if (cursor_pos - press_pos).length() >= CLICK_DRAG_THRESHOLD_PX {
                state.phase = MouseInteractionPhase::Dragging;
            }
        }
    }

    if just_released {
        state.phase = match state.phase {
            MouseInteractionPhase::PendingClassification => {
                MouseInteractionPhase::Clicked(state.press_position.unwrap_or_default())
            }
            _ => {
                state.press_position = None;
                MouseInteractionPhase::Idle
            }
        };
        return;
    }

    // Fallback: if button is no longer pressed and we're in an active state, reset.
    // This handles edge cases like losing focus or the cursor leaving the window.
    if !pressed
        && !matches!(
            state.phase,
            MouseInteractionPhase::Idle | MouseInteractionPhase::Clicked(_)
        )
    {
        state.phase = MouseInteractionPhase::Idle;
        state.press_position = None;
    }
}

pub fn classify_mouse_interaction(
    mut state: ResMut<MouseInteractionState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let cursor_position = window.cursor_position();

    classify_button(
        &mut state.left,
        mouse_buttons.just_pressed(MouseButton::Left),
        mouse_buttons.pressed(MouseButton::Left),
        mouse_buttons.just_released(MouseButton::Left),
        cursor_position,
    );

    classify_button(
        &mut state.right,
        mouse_buttons.just_pressed(MouseButton::Right),
        mouse_buttons.pressed(MouseButton::Right),
        mouse_buttons.just_released(MouseButton::Right),
        cursor_position,
    );
}

pub struct MouseInteractionPlugin;

impl Plugin for MouseInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseInteractionState>()
            .add_systems(PreUpdate, classify_mouse_interaction);
    }
}
