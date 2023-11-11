use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}


// #[derive(Serialize, Deserialize, Debug)]
// pub struct TestStruct {
//     name: String,
//     size: i32;
// }
