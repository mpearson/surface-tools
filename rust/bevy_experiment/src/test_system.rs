// use bevy::math;
// use std::env;
use bevy::prelude::*;

use crate::test_component::MapTile;
use crate::test_component::PositionGeodetic;
// mod test2;
// mod test3;
// mod test4;

// fn main() {
//     println!("wow, such bevy");
//     println!("very webGPU");
//     println!("much borrow check");
//     println!("wow");

//     App::new().run();

//     // let args: Vec<String> = env::args().collect();
//     // if args.len() < 2 {
//     //     println!("No test specified");
//     //     return;
//     // }

//     // let test_number = &args[1];

//     // match (test_number).trim() {
//     //     "1" => test1::guess_numbers(),
//     //     "2" => test2::test_search_array(),
//     //     "3" => test3::test_fibonacci(),
//     //     "4" => test4::do_stuff(),
//     //     _ => println!("No test specified"),
//     // }
// }

pub fn init_map(mut commands: Commands) {
    println!("creating initial map tiles");
    commands.spawn((
        MapTile,
        PositionGeodetic {
            lat_deg: 36.876703200359209,
            lon_deg: -121.7959200017972,
            height: 0.0,
        },
    ));
    commands.spawn((
        MapTile,
        PositionGeodetic {
            lat_deg: 37.876703200359209,
            lon_deg: -121.7959200017972,
            height: 0.0,
        },
    ));
    // MapTile
    // PositionGeodetic
}

pub fn print_map_tiles(query: Query<&PositionGeodetic, With<MapTile>>) {
    println!("all tiles:");
    for pos in query.iter() {
        println!("  {}, {}", pos.lat_deg, pos.lon_deg);
    }
}

pub fn update_tiles(mut query: Query<&mut PositionGeodetic, With<MapTile>>) {
    println!("updating tiles");
    for mut pos in &mut query {
        pos.lat_deg *= std::f64::consts::PI / 180.0;
        pos.lon_deg *= std::f64::consts::PI / 180.0;
    }
}
