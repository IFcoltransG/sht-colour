use num::{checked_pow, rational::Ratio, CheckedMul, Integer, Unsigned};
use std::{
    fmt::{Display, Formatter, Result as FMTResult, UpperHex},
    str::FromStr,
};

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParseHexError {
    EmptyCode,
    MissingOctothorpe,
    InvalidDigitCount,
    DigitParseError,
    Overflow,
}

#[derive(Debug, PartialEq, Clone)]
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

    pub fn components(&self) -> (Ratio<T>, Ratio<T>, Ratio<T>) {
        let Self { red, green, blue } = self;
        (red.clone(), green.clone(), blue.clone())
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

impl<T> Display for RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul + From<u8> + UpperHex,
{
    fn fmt(&self, formatter: &mut Formatter) -> FMTResult {
        let width = formatter.width().unwrap_or(2);
        let denominator = checked_pow(<T>::from(16), width).unwrap() - <T>::one();

        let from_ratio = |ratio: Ratio<T>| {
            ratio
                .checked_mul(&Ratio::from_integer(denominator.clone()))
                .unwrap()
        };

        let (red, green, blue) = self.clone().components();
        write!(
            formatter,
            "#{:0width$X}{:0width$X}{:0width$X}",
            from_ratio(red).to_integer(),
            from_ratio(green).to_integer(),
            from_ratio(blue).to_integer(),
            width = width
        )
    }
}

fn channel_split(s: &str) -> (&str, &str, &str) {
    let first = s.len() / 3;
    let second = first * 2;
    (&s[..first], &s[first..second], &s[second..])
}

fn parse_channel<T>(digits: &str) -> Result<Ratio<T>, ParseHexError>
where
    T: Unsigned + Integer + FromStr + Clone + CheckedMul,
    u8: Into<T>,
{
    Ok(<Ratio<T>>::new(
        T::from_str_radix(digits, 16).map_err(|_| ParseHexError::DigitParseError)?,
        checked_pow(2.into(), digits.len() * 4).ok_or(ParseHexError::Overflow)? - T::one(),
    ))
}

#[cfg(test)]
mod tests;
