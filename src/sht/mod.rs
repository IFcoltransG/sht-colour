use nom::error::Error;
use num::{
    pow::Pow, rational::Ratio, CheckedAdd, CheckedDiv, CheckedMul, Integer, One, Unsigned, Zero,
};
use parser::parse_sht;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParsePropertyError {
    ValueErrors(Vec<SHTValueError>),
    ParseFailure(Error<String>),
    InputRemaining(String),
}

impl From<Error<&str>> for ParsePropertyError {
    fn from(value: Error<&str>) -> Self {
        match value {
            Error { input, code } => {
                ParsePropertyError::ParseFailure(Error::new(input.to_owned(), code))
            }
        }
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
// primary set yet shade 0 or tint 1
// direction equal to primary
// blend 0 or 1

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

#[derive(Debug, PartialEq)]
pub struct SHT<T: Clone + Integer + Unsigned> {
    channel_ratios: ChannelRatios<T>,
    shade: Ratio<T>, // None=1
    tint: Ratio<T>,  // None=0
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

    fn normal(self: Self) -> Result<Self, Vec<SHTValueError>> {
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
                //colour has two brightest channels
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
                tint,
                shade,
            })
        } else {
            Err(errors)
        }
    }
}
//tint out of bounds, shade out of bounds
impl<T> FromStr for SHT<T>
where
    T: Clone
        + Integer
        + Unsigned
        + FromStr
        + From<u8>
        + CheckedMul
        + CheckedAdd
        + CheckedDiv
        + Pow<T, Output = T>,
{
    type Err = ParsePropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_sht(s)
    }
}

#[cfg(test)]
mod tests;

mod parser;
