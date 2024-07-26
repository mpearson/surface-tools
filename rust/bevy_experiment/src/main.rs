// use bevy::ecs::schedule::IntoSystemConfigs;
// use std::env;
// use bevy::prelude::*;
use bevy::prelude::App;
use bevy::prelude::DefaultPlugins;
// use bevy::prelude::Startup;
// use bevy::prelude::Update;

mod basic_scene;
mod camera;
mod orbit_camera;
mod test_component;
mod test_plugin;
mod test_system;

fn main() {
    // println!("wow, such bevy");
    // println!("very webGPU");
    // println!("much borrow check");
    // println!("wow");

    // let systems = (test_system::update_tiles, test_system::print_map_tiles).chain();

    App::new()
        .add_plugins((DefaultPlugins, test_plugin::MyFirstPlugin))
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
