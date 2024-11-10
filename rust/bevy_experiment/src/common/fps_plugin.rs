use bevy::prelude::{default, App, Commands, Plugin, Startup};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use iyes_perf_ui::prelude::*;

pub struct FpsCounterPlugin;

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PerfUiPlugin, FrameTimeDiagnosticsPlugin::default()))
            .add_systems(Startup, setup_fps_counter);
    }
}

fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..default()
        },
        // PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}
