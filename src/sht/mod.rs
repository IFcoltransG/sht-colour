use nom::error::Error;
use num::{
    checked_pow, rational::Ratio, CheckedAdd, CheckedDiv, CheckedMul, Integer, One, Unsigned, Zero,
};
use parser::parse_sht;
use std::{
    convert::TryInto,
    fmt::{Display, Formatter, Result as FMTResult},
    ops::{Div, Rem},
    str::FromStr,
};

#[derive(Debug, PartialEq, Clone)]
pub struct SHT<T: Clone + Integer + Unsigned> {
    channel_ratios: ChannelRatios<T>,
    shade: Ratio<T>, // None=1
    tint: Ratio<T>,  // None=0
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelRatios<T: Clone + Integer + Unsigned> {
    OneBrightestChannel {
        primary: ColourChannel,
        direction_blend: Option<(ColourChannel, Ratio<T>)>,
    },
    TwoBrightestChannels {
        secondary: SecondaryColour,
    },
    ThreeBrightestChannels,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColourChannel {
    Red,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecondaryColour {
    Cyan,
    Yellow,
    Magenta,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParsePropertyError {
    ValueErrors(Vec<SHTValueError>),
    ParseFailure(Error<String>),
    InputRemaining(String),
}

impl From<Error<&str>> for ParsePropertyError {
    fn from(value: Error<&str>) -> Self {
        let Error { input, code } = value;
        ParsePropertyError::ParseFailure(Error::new(input.to_owned(), code))
    }
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SHTValueError {
    PrimaryShadeZero,       // primary set with shade set to 0
    PrimaryTintOne,         // primary set with tint set to 0
    SecondaryShadeZero,     // secondary set with shade set to 0
    SecondaryTintOne,       // secondary set with shad set to 0
    DirectionEqualsPrimary, // direction equal to primary
    ValueOutOfBounds,       // a ratio is not in 0..1 range
    BlendZero,              // blend set to 0
    BlendOne,               // blend set to 1
}

impl<T: Clone + Integer + Unsigned> SHT<T> {
    pub fn new(
        channel_ratios: ChannelRatios<T>,
        shade: Ratio<T>,
        tint: Ratio<T>,
    ) -> Result<Self, Vec<SHTValueError>> {
        let code = SHT {
            channel_ratios,
            shade,
            tint,
        };
        match code.normal() {
            Ok(code) => Ok(code),
            Err(errs) => Err(errs),
        }
    }

    pub fn components(&self) -> (ChannelRatios<T>, Ratio<T>, Ratio<T>) {
        let Self {
            channel_ratios,
            shade,
            tint,
        } = self;
        (channel_ratios.clone(), shade.clone(), tint.clone())
    }

    fn normal(self) -> Result<Self, Vec<SHTValueError>> {
        let Self {
            channel_ratios,
            shade,
            tint,
        } = self;
        // validate fields:
        let mut errors = Vec::with_capacity(16); // more than strictly needed
        match channel_ratios.clone() {
            ChannelRatios::OneBrightestChannel {
                primary,
                direction_blend,
            } => {
                // colour has one brightest channel
                if shade.is_zero() {
                    errors.push(SHTValueError::PrimaryShadeZero);
                }
                if tint.is_one() {
                    errors.push(SHTValueError::PrimaryTintOne);
                }
                if let Some((direction, blend)) = direction_blend {
                    // colour has a second-brightest channel
                    if direction == primary {
                        errors.push(SHTValueError::DirectionEqualsPrimary);
                    }
                    if blend.is_zero() {
                        errors.push(SHTValueError::BlendZero);
                    }
                    if blend.is_one() {
                        errors.push(SHTValueError::BlendOne);
                    }
                    if blend > Ratio::one() {
                        errors.push(SHTValueError::ValueOutOfBounds);
                    }
                }
            }
            ChannelRatios::TwoBrightestChannels { .. } => {
                // colour has two brightest channels
                if shade.is_zero() {
                    errors.push(SHTValueError::SecondaryShadeZero);
                }
                if tint.is_one() {
                    errors.push(SHTValueError::SecondaryTintOne);
                }
            }
            ChannelRatios::ThreeBrightestChannels => {}
        }
        if tint > Ratio::one() {
            errors.push(SHTValueError::ValueOutOfBounds);
        }
        if shade > Ratio::one() {
            errors.push(SHTValueError::ValueOutOfBounds);
        }
        if errors.is_empty() {
            Ok(Self {
                channel_ratios,
                shade,
                tint,
            })
        } else {
            Err(errors)
        }
    }
}

impl<T> FromStr for SHT<T>
where
    T: Clone + Integer + Unsigned + FromStr + CheckedMul + CheckedAdd + CheckedDiv,
    u8: Into<T>,
{
    type Err = ParsePropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_sht(s)
    }
}

/// Possibly rounds a base 12 number
/// If round_up, adds 1 to the number
/// Othewise, leaves number unchanged
/// Number is a slice of u8 digits
fn round(input: &[u8], round_up: bool) -> Vec<u8> {
    eprintln!("Rounding: {}", round_up);
    if round_up {
        if let Some((&last, rest)) = input.split_last() {
            let rounded_last = last + 1;
            if rounded_last >= 12 {
                round(rest, round_up)
            } else {
                let mut mut_rest = rest.to_vec();
                mut_rest.push(rounded_last);
                mut_rest
            }
        } else {
            vec![12]
        }
    } else {
        input.to_vec()
    }
}


/// Converts a ratio to a fixed point base 12 string
fn duodecimal<T>(mut input: Ratio<T>, precision: usize) -> String
where
    T: TryInto<usize> + Integer + Zero + Rem<T, Output = T> + Div<T, Output = T> + Clone,
    u8: Into<T>,
{
    let half = || Ratio::new(1.into(), 2.into());
    let digit_characters = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'X', 'E'];
    let mut digits = Vec::with_capacity(precision);
    if input >= <_>::one() {
        return "W".to_string();
    }
    let mut round_up = false;
    for digits_left in (0..precision).rev() {
        let scaled = input * Ratio::from_integer(12.into());
        input = scaled.fract();
        if digits_left.is_zero() {
            // round because no more digits
            // comparing remainder to 0.5
            round_up = input >= half();
        }
        let integer_part = scaled.to_integer();
        let next_digit = match integer_part.try_into() {
            Ok(n) if n < 12 => n as u8,
            _ => 12u8,
        };
        digits.push(next_digit);
        if input.is_zero() {
            break;
        }
    }
    // possibly round up, then convert &[u8] to digit String
    round(&digits, round_up)
        .iter()
        .map(|&c| digit_characters.get(c as usize).unwrap_or(&'W'))
        .collect()
}

impl<T> Display for SHT<T>
where
    T: TryInto<usize> + Unsigned + Integer + Clone + Display + One,
    u8: Into<T>,
{
    fn fmt(&self, formatter: &mut Formatter) -> FMTResult {
        let precision = formatter.precision().unwrap_or(2);

        let ratio_to_str = |ratio: Ratio<T>| duodecimal(ratio, precision);
        let primary_to_str = |primary| match primary {
            ColourChannel::Red => "r".to_string(),
            ColourChannel::Green => "g".to_string(),
            ColourChannel::Blue => "b".to_string(),
        };
        let secondary_to_str = |secondary| match secondary {
            SecondaryColour::Cyan => "c".to_string(),
            SecondaryColour::Yellow => "y".to_string(),
            SecondaryColour::Magenta => "m".to_string(),
        };

        let (channel_ratios, shade_ratio, tint_ratio) = self.clone().components();
        let tint = (!tint_ratio.is_zero()).then(|| tint_ratio);
        let shade = (!shade_ratio.is_one()).then(|| shade_ratio);
        let (primary, secondary, direction, blend) = match channel_ratios {
            ChannelRatios::OneBrightestChannel {
                primary,
                direction_blend,
            } => {
                if let Some((direction, blend)) = direction_blend {
                    (Some(primary), None, Some(direction), Some(blend))
                } else {
                    (Some(primary), None, None, None)
                }
            }
            ChannelRatios::TwoBrightestChannels { secondary } => {
                (None, Some(secondary), None, None)
            }
            ChannelRatios::ThreeBrightestChannels => (None, None, None, None),
        };
        write!(
            formatter,
            "{}{}{}{}{}{}",
            shade.map(ratio_to_str).unwrap_or_else(String::new),
            primary.map(primary_to_str).unwrap_or_else(String::new),
            blend.map(ratio_to_str).unwrap_or_else(String::new),
            direction.map(primary_to_str).unwrap_or_else(String::new),
            secondary.map(secondary_to_str).unwrap_or_else(String::new),
            tint.map(ratio_to_str).unwrap_or_else(String::new)
        )
    }
}

#[cfg(test)]
mod tests;

mod parser;
