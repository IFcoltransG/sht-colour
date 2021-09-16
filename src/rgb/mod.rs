// use itertools::Itertools;
// use nom;
use num::{rational::Ratio, CheckedMul, Integer, Unsigned};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[non_exhaustive]

pub enum ParseHexError {
    EmptyCode,
    MissingOctothorpe,
    InvalidDigitCount,
    DigitParseError,
    Overflow,
}

#[derive(Debug, PartialEq)]

pub struct RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    red: Ratio<T>,
    green: Ratio<T>,
    blue: Ratio<T>,
}

impl<T> RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    pub fn new(red: Ratio<T>, green: Ratio<T>, blue: Ratio<T>) -> RGB<T> {
        RGB { red, green, blue }
    }

    pub fn components(self) -> (Ratio<T>, Ratio<T>, Ratio<T>) {
        let Self { red, green, blue } = self;
        (red, green, blue)
    }
}

impl<T> FromStr for RGB<T>
where
    T: Unsigned + Integer + FromStr + From<u8> + Clone + CheckedMul,
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

fn parse_channel<T>(digits: &str) -> Result<Ratio<T>, ParseHexError>
where
    T: Unsigned + Integer + FromStr + From<u8> + Clone + CheckedMul,
{
    Ok(<Ratio<T>>::new(
        T::from_str_radix(digits, 16).map_err(|_| ParseHexError::DigitParseError)?,
        {
            let mut base = T::one();
            for _ in 0..(digits.len() * 4) {
                base = base.checked_mul(&2.into()).ok_or(ParseHexError::Overflow)?;
            }
            base
        } - T::one(),
    ))
}

#[cfg(test)]
mod tests;
