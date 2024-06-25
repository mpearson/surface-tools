// use bevy::math;
// use std::env;
// use bevy::prelude::*;
use bevy::prelude::{App, Plugin, Startup, Update};

// use crate::test_component::MapTile;
// use crate::test_component::PositionGeodetic;
use crate::camera;
use crate::test_system;

pub struct MyFirstPlugin;

impl Plugin for MyFirstPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                camera::spawn_stuff,
                test_system::init_map,
                test_system::update_tiles,
            ),
        )
        // .add_systems(Startup, test_system::print_map_tiles)
        .add_systems(Update, test_system::print_map_tiles);
    }
}
