use std::env;

mod test1;
mod test2;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No test specified");
        return;
    }

    match (&args[1]).trim() {
        "1" => test1::guess_numbers(),
        "2" => test2::test_search_array(),
        _ => println!("No test specified"),
    }
}
