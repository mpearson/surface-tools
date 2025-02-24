// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::prelude::*;

// use crate::test_component::MapTile;
// use crate::test_component::PositionGeodetic;
use crate::basic_scene;

use crate::test_system;

// use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
// use iyes_perf_ui::prelude::*;

pub struct SpinningCubePlugin;

impl Plugin for SpinningCubePlugin {
    fn build(&self, app: &mut App) {
        app
            // app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(
                Startup,
                (
                    basic_scene::spawn_spinning_cube,
                    test_system::init_map,
                    test_system::update_tiles,
                ),
            )
            .add_systems(Update, (basic_scene::rotate_cube,));
        // .add_systems(Startup, test_system::print_map_tiles)
        // .add_systems(Update, test_system::print_map_tiles);
    }
}
