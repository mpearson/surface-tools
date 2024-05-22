fn fibonacci(n: i32) -> i32 {
    if n < 3 {
        return 1;
    }

    let mut prev_value = 1;
    let mut current_value = 1;

    for _ in 2..n {
        let new_value = current_value + prev_value;
        prev_value = current_value;
        current_value = new_value;
    }
    return current_value;
}

pub fn test_fibonacci() {
    for n in 1..10 {
        println!("{}: {}", n, fibonacci(n));
    }
}
