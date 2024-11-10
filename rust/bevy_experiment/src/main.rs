// use bevy::ecs::schedule::IntoSystemConfigs;
// use std::env;
// use bevy::prelude::*;
use bevy::prelude::default;
use bevy::prelude::App;
use bevy::prelude::Commands;
use bevy::prelude::DefaultPlugins;
use bevy::prelude::Startup;
// use bevy::prelude::Update;

mod basic_scene;
mod camera;
mod common;
mod test_component;
mod test_plugin;
mod test_system;

// mod orbit_camera;
// use crate::orbit_camera::OrbitCameraPlugin;

// use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
// use iyes_perf_ui::prelude::*;

fn main() {
    // println!("wow, such bevy");
    // println!("very webGPU");
    // println!("much borrow check");
    // println!("wow");

    // let systems = (test_system::update_tiles, test_system::print_map_tiles).chain();

    App::new()
        .add_plugins((
            DefaultPlugins,
            test_plugin::MyFirstPlugin,
            common::fps_plugin::FpsCounterPlugin,
            // orbit_camera::plugin::OrbitCameraPlugin,
            // PerfUiPlugin,
            // FrameTimeDiagnosticsPlugin::default(),
        ))
        // .add_systems(Startup, setup_fps_counter)
        .run();

    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("No test specified");
    //     return;
    // }

    // let test_number = &args[1];

    // match (test_number).trim() {
    //     "1" => test1::guess_numbers(),
    //     "2" => test2::test_search_array(),
    //     "3" => test3::test_fibonacci(),
    //     "4" => test4::do_stuff(),
    //     _ => println!("No test specified"),
    // }
}

// fn setup_fps_counter(mut commands: Commands) {
//     // spawn a camera to be able to see anything
//     // commands.spawn(Camera2dBundle::default());

//     // create a simple Perf UI with default settings
//     // and all entries provided by the crate:
//     // commands.spawn(PerfUiCompleteBundle::default());
//     commands.spawn((
//         PerfUiRoot {
//             display_labels: false,
//             layout_horizontal: true,
//             ..default()
//         },
//         // PerfUiEntryFPSWorst::default(),
//         PerfUiEntryFPS::default(),
//     ));
// }
