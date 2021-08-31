use num::{Integer, Unsigned};
//use unicode_segmentation::UnicodeSegmentation;

mod rgb;
mod sht;

pub fn sht_to_rgb<N, T>(_input: sht::SHT<N>) -> rgb::RGB<T>
where
    N: Clone + Integer + Unsigned,
    T: Integer + Unsigned,
{
    todo!()
}
pub fn rgb_to_sht<T, N>(_input: rgb::RGB<T>) -> sht::SHT<N>
where
    T: Integer + Unsigned,
    N: Clone + Integer + Unsigned,
{
    todo!()
}
pub fn parse_sht<T: Clone + Integer + Unsigned>(_input: &str) -> Option<sht::SHT<T>> {
    None
}
pub fn parse_rgb<T: Integer + Unsigned>(_input: &str) -> Option<rgb::RGB<T>> {
    None
}
