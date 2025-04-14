use bevy::prelude::*;

mod basic_scene;
mod common;
mod orbit_camera;

fn main() {
    println!("wow, such bevy");
    println!("very webGPU");
    println!("much borrow check");
    println!("wow");

    App::new()
        .add_plugins((
            DefaultPlugins,
            basic_scene::BasicScenePlugin,
            common::fps_plugin::FpsCounterPlugin,
            orbit_camera::plugin::OrbitCameraPlugin,
        ))
        .run();
}
