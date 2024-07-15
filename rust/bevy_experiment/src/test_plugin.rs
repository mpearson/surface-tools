// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::prelude::*;

// use crate::test_component::MapTile;
// use crate::test_component::PositionGeodetic;
use crate::basic_scene;
use crate::camera::{pan_orbit_camera, PanOrbitState};
use crate::test_system;

pub struct MyFirstPlugin;

impl Plugin for MyFirstPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                basic_scene::spawn_stuff,
                test_system::init_map,
                test_system::update_tiles,
            ),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(any_with_component::<PanOrbitState>),
        );
        // .add_systems(Startup, test_system::print_map_tiles)
        // .add_systems(Update, test_system::print_map_tiles);
    }
}
