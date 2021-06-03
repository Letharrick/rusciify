use std::ops::{Index, Mul};

use image::{Primitive, Luma};

#[derive(Debug)]
pub struct CharacterMap(Vec<char>);

impl Default for CharacterMap {
    fn default() -> Self {
        Self(vec![' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'])
    }
}

impl<S: AsRef<str>> From<S> for CharacterMap {
    fn from(string: S) -> Self {
        Self(string.as_ref().chars().collect())
    }
}

impl<T: Primitive + Mul<Output = T>> Index<Luma<T>> for CharacterMap {
    type Output = char;

    fn index(&self, luma_pixel: Luma<T>) -> &Self::Output {
        let index = (luma_pixel[0].to_usize().unwrap() as f32 * self.0.len() as f32 / (u8::MAX as f32 + 1.0).floor()) as usize;

        &self.0[index]
    }
}