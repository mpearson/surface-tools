// use std::env;
// use bevy::prelude::*;
use bevy::prelude::Component;

// mod test1;
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

#[derive(Component)]
pub struct MapTile;

#[derive(Component)]
pub struct PositionGeodetic {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub height: f64,
}
