use std::collections::HashMap;
use std::ops::{Add, Div};

fn main() {
    let v = vec![1, 2, 3, 4, 5];
    let a = aver(&v);
    println!("{}", a);
}

fn aver<T>(vec: &[T]) -> T
where
    T: Add<Output = T> + Div<usize, Output = T> + Default + Copy,
{
    let mut total: T = T::default();

    for &i in vec {
        total = total + i;
    }

    total / vec.len()
}
