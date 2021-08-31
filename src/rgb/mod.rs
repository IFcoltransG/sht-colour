//use itertools::Itertools;
//use nom;
use num::{Integer, Unsigned};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParseHexError {
    EmptyCode,
    MissingOctothorpe,
    InvalidDigitCount,
    DigitParseError,
}

#[derive(Debug, PartialEq)]
pub struct RGB<T: Unsigned + Integer> {
    red: T,
    green: T,
    blue: T,
}
impl<T> RGB<T>
where
    T: Unsigned + Integer + FromStr,
{
    pub fn new(red: T, green: T, blue: T) -> RGB<T> {
        RGB { red, green, blue }
    }
}
impl<T> FromStr for RGB<T>
where
    T: Unsigned + Integer + FromStr,
{
    type Err = ParseHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseHexError::EmptyCode);
        }
        if &s[..1] != "#" {
            return Err(ParseHexError::MissingOctothorpe);
        }
        let digits = &s[1..];
        if digits.len() % 3 != 0 {
            return Err(ParseHexError::InvalidDigitCount);
        }
        let (red_digits, green_digits, blue_digits) = channel_split(digits);
        let (red, green, blue) = (
            parse_channel(red_digits)?,
            parse_channel(green_digits)?,
            parse_channel(blue_digits)?,
        );
        Ok(RGB::new(red, green, blue))
    }
}

fn channel_split(s: &str) -> (&str, &str, &str) {
    let first = s.len() / 3;
    let second = first * 2;
    (&s[..first], &s[first..second], &s[second..])
}

fn parse_channel<T>(digits: &str) -> Result<T, ParseHexError>
where
    T: Unsigned + Integer + FromStr,
{
    T::from_str_radix(digits, 16).map_err(|_| ParseHexError::DigitParseError)
}

#[cfg(test)]
mod tests;
