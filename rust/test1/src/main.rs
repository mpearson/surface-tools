use std::io;
use std::cmp::Ordering;
use rand::Rng;

fn main() {
    // println!("What is your favorite color?");
    println!("Guess a number between 1 and 100:");

    // let correct_color = "blue";
    let correct_number: i32 = rand::thread_rng().gen_range(1..=100);


    let mut guess: String = String::new();
    loop {
        guess.truncate(0);
        let result = io::stdin().read_line(&mut guess);
        result.expect("wtf man");
        let guess: i32 = match guess.trim().parse() {

            Ok(num) => num,
            Err(_) => continue,
        };
            // .expect("that aint no number hoss");

        // let mut color: String = String::new();
        // io::stdin().read_line(&mut color);
        // let result: Result<usize, io::Error> = io::stdin().read_line(&mut color);
        // color = color.strip_suffix("\n").unwrap().to_string();
        // println!("{color}");
        // let mut derp: &str = "derp";
        // derp.chars().nth(2).
        // println!("{derp}");

        match guess.cmp(&correct_number) {
            Ordering::Less => println!(">"),
            Ordering::Greater => println!("<"),
            Ordering::Equal => {
                println!("yay");
                break;
            },
        }
    }


    // if correct_color.eq(&color) {
    //     println!("You chose....wisely.");
    // } else {
    //     println!("You chose....poorly.");
    // }
}
