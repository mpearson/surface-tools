



pub fn enhance_further(s: &mut String) {
    println!("Enhancing further {}", s);

    s.push_str("1");
}


pub fn enhance(s: &mut String) {
    println!("Enhancing {}", s);



    s.push_str("!!!");

    enhance_further(s);
}



pub fn do_stuff() {

    let x = 5;
    let y = &x;

    println!("x: {}, y: {}", x, y);

    let mut s = String::from("wat");

    let substr = s[s.len() - 2..].to_owned();

    enhance(&mut s);



    s += "22222";

    println!("substr: {}", substr);

    println!("Enhanced string: {}", s);

}
