use std::env;

mod test1;
mod test2;
mod test3;
mod test4;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No test specified");
        return;
    }

    let test_number = &args[1];

    match (test_number).trim() {
        "1" => test1::guess_numbers(),
        "2" => test2::test_search_array(),
        "3" => test3::test_fibonacci(),
        "4" => test4::do_stuff(),
        _ => println!("No test specified"),
    }
}
