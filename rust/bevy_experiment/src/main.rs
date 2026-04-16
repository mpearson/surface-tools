use bevy::prelude::*;

mod basic_scene;
mod common;
mod orbit_camera;
mod polygon_tool;

fn main() {
    println!("wow, such bevy");
    println!("very webGPU");
    println!("much borrow check");
    println!("wow");
    App::new()
        .add_plugins((
            DefaultPlugins,
            basic_scene::BasicScenePlugin,
            // common::fps_plugin::FpsCounterPlugin,
            common::mouse_interaction::MouseInteractionPlugin,
            orbit_camera::plugin::OrbitCameraPlugin,
            polygon_tool::plugin::PolygonToolPlugin,
        ))
        .run();
}
