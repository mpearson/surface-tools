use std::io;
use std::option::Option;

// This demonstrates that when you create a slice with `0..get_length(&array)`, get_length() is
// only called once.
fn get_length(array: &[i32]) -> usize {
    println!("getting length");
    array.len()
}

// Example of a function that returns an optional value, and accepts
// array and int references as arguments.
fn find_thingy(array: &[i32], divisor: &i32) -> Option<(usize, i32)> {
    for i in 0..get_length(&array) {
        if array[i] % divisor == 0 {
            return Option::Some((i, array[i]));
        }
    }
    Option::None
}

// Example of a function that modifies an int variable in place, using a mutable reference argument.
fn improve_number(n: &mut i32) {
    *n += 1;
}

fn test_array_search() {
    // println!("What is your favorite color?");
    println!("pick a nubmer or something");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("dangit!");
    let mut divisor = buf.trim().parse::<i32>().expect("double dangit!");

    improve_number(&mut divisor);

    let a = [2, 4, 5, 15, 21, 68, 54, 1];

    let result = find_thingy(&a, &divisor);

    match result {
        Option::Some((i, b)) => println!("found {b} at index {i}"),
        Option::None => println!("we aint found shit"),
    }
    // println!("found {b} at index {i}");
}

fn main() {
    test_array_search();
}
