use std::collections::HashMap;
use std::ops::{Add, Div};

fn main() {
    let text = "A super long string which should be wrapped at some point because there is not enough space to do the whole thing.".to_string();
    let wrapped_text = textwrap::wrap(&text, 20);
    println!("{:?}", wrapped_text);
}
