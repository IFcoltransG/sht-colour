use nom::error::Error;
use num::{rational::Ratio, CheckedAdd, CheckedDiv, CheckedMul, Integer, One, Unsigned, Zero};
use parser::parse_sht;
use std::{
    convert::TryInto,
    fmt::{Display, Formatter, Result as FMTResult},
    ops::{Div, Rem},
    str::FromStr,
};

/// A representation of a colour in [SHT format](https://omaitzen.com/sht/).
///
/// The SHT colour format is intended to be human-readable and human-writable.
/// For instance, the code for the colour red is `"r"`, and the code for a dark
/// yellow is `"3y"`. SHT codes cover the same colour space as RGB codes, but
/// map commonly used colours onto memorable strings.
///
/// Extra precision can usually be expressed by appending characters to an
/// existing code. For instance, darkening the code for red is achieved by
/// adding a digit to the start, `"9r"`, and `"9r4g"` is the same colour but
/// with a hint of green.
///
/// See the [`Display` implementation] for more details on the format.
///
/// # Examples
/// ```
/// use num::rational::Ratio;
/// use sht_colour::sht::{
///     ChannelRatios::OneBrightestChannel,
///     ColourChannel::{Green, Red},
///     SHT,
/// };
///
/// // Quite red, a bit green, slightly faded
/// let code = "8r6g3";
///
/// // Parse a colour from a string
/// let parsed_colour = code.parse::<SHT<u8>>().unwrap();
///
/// // Construct a colour manually
/// let shade = Ratio::new(8, 12);
/// let tint = Ratio::new(3, 12);
/// let blend = Ratio::new(6, 12);
/// let constructed_colour = SHT::new(
///     OneBrightestChannel {
///         primary: Red,
///         direction_blend: Some((Green, blend)),
///     },
///     shade,
///     tint,
/// )
/// .unwrap();
///
/// // Both colours are the same
/// assert_eq!(constructed_colour, parsed_colour);
/// // The colour's string representation is the same as the original string
/// assert_eq!(constructed_colour.to_string(), code);
/// ```
///
/// [`Display` implementation]: SHT#impl-Display
#[derive(Debug, PartialEq, Clone)]
pub struct SHT<T: Clone + Integer + Unsigned> {
    /// [`ChannelRatios`] value representing the relative strength of colour
    /// components in the SHT.
    channel_ratios: ChannelRatios<T>,
    /// Overall brightness, measured as strength of strongest colour channel
    /// relative to weakest.
    ///
    /// Has a default value of 1 if unspecified.
    shade: Ratio<T>,
    /// Lightness, equal to strength of weakest channel.
    ///
    /// Has a default value of 0 if unspecified.
    tint: Ratio<T>,
}

/// Part of an [`SHT`] value, representing data about hues and relative strength
/// of channels
///
/// # Example
/// ```
/// use num::rational::Ratio;
/// use sht_colour::sht::{ChannelRatios::ThreeBrightestChannels, SHT};
///
/// let colour = "W".parse::<SHT<_>>().unwrap();
///
/// let channel_ratios = ThreeBrightestChannels;
/// let colour_components = (
///     channel_ratios,
///     Ratio::from_integer(1_u8),
///     Ratio::from_integer(1_u8),
/// );
///
/// assert_eq!(colour.components(), colour_components);
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelRatios<T: Clone + Integer + Unsigned> {
    /// Represents colours where one channel (either [red], [blue] or [green])
    /// is strictly brighter than the other two.
    ///
    /// [red]: ColourChannel::Red
    /// [green]: ColourChannel::Green
    /// [blue]: ColourChannel::Blue
    OneBrightestChannel {
        /// Stores whichever colour channel is brightest.
        primary: ColourChannel,
        /// If all three channels have different brightnesses, then this field
        /// contains whichever channel is *second* brightest, as well as a
        /// ratio: the brightness of the second brightest channel divided by the
        /// brightness of the brightest channel. (Both the colour channel and
        /// its relative strength stored in a tuple.)
        ///
        /// Otherwise, if channels other than the brightest channel are equal to
        /// each other, this field is `None`.
        direction_blend: Option<(ColourChannel, Ratio<T>)>,
    },
    /// Represents colours where two channels (from among [red], [blue] or
    /// [green]) have the same brightness as each other and have strictly
    /// greater brightness than the other channel.
    ///
    /// [red]: ColourChannel::Red
    /// [green]: ColourChannel::Green
    /// [blue]: ColourChannel::Blue
    TwoBrightestChannels {
        /// Holds the secondary colour (either [cyan], [yellow] or [magenta])
        /// that represents whichever combination of two [primary colour
        /// channels] are brightest.
        ///
        /// [primary colour channels]: ColourChannel
        /// [cyan]: SecondaryColour::Cyan
        /// [yellow]: SecondaryColour::Yellow
        /// [magenta]: SecondaryColour::Magenta
        secondary: SecondaryColour,
    },
    /// Represents colours where all three channels ([red], [blue] and [green])
    /// have the exact same brightness as each other.
    ///
    /// [red]: ColourChannel::Red
    /// [green]: ColourChannel::Green
    /// [blue]: ColourChannel::Blue
    ThreeBrightestChannels,
}

/// Represents a primary colour (using additive mixing).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColourChannel {
    /// The colour red.
    Red,
    /// The colour green.
    Green,
    /// The colour blue.
    Blue,
}

/// Represents a secondary colour (using additive mixing).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecondaryColour {
    /// The colour cyan, made of green and blue.
    Cyan,
    /// The colour yellow, made of red and green.
    Yellow,
    /// The colour magenta, made of red and blue.
    Magenta,
}

/// Represents possible errors parsing an [`SHT`] from a string.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParsePropertyError {
    /// Parsed data, but failed to construct an [`SHT`] from it.
    ValueErrors(Vec<SHTValueError>),
    /// Could not parse data from the string.
    ParseFailure(Error<String>),
    /// Parsed data from the string, but with leftover unparsed characters.
    InputRemaining(String),
}

impl From<Error<&str>> for ParsePropertyError {
    fn from(value: Error<&str>) -> Self {
        let Error { input, code } = value;
        ParsePropertyError::ParseFailure(Error::new(input.to_owned(), code))
    }
}

/// Represents possible errors when constructing an [`SHT`] from component
/// values.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SHTValueError {
    /// `primary` set, while `shade` set
    /// to 0.
    PrimaryShadeZero,
    /// `primary` set, while `tint` set to
    /// 0.
    PrimaryTintOne,
    /// `secondary` set, while `shade` set
    /// to 0.
    SecondaryShadeZero,
    /// `secondary` set, while `tint` set
    /// to 1.
    SecondaryTintOne,
    /// `direction` is equal to `primary`.
    DirectionEqualsPrimary,
    /// A [ratio](num::rational::Ratio) is not in `0..1` range
    /// (inclusive).
    ValueOutOfBounds,
    /// `blend` set to 0.
    BlendZero,
    /// `blend` set to 1.
    BlendOne,
}

impl<T: Clone + Integer + Unsigned> SHT<T> {
    /// Constructs an [`SHT`] value.
    ///
    /// # Arguments
    ///
    /// * `channel_ratios` - [`ChannelRatios`] value representing the relative
    ///   strength of colour components in the SHT.
    /// * `shade` - Overall brightness, measured as strength of strongest colour
    ///   channel relative to weakest.
    /// * `tint` - Lightness, equal to strength of weakest channel.
    ///
    /// # Example
    /// ```
    /// use num::rational::Ratio;
    /// use sht_colour::sht::{ChannelRatios::OneBrightestChannel, ColourChannel::Red, SHT};
    ///
    /// let red_ratio = OneBrightestChannel {
    ///     primary: Red,
    ///     direction_blend: None,
    /// };
    /// let dark_red = <SHT<u8>>::new(red_ratio, Ratio::new(4, 12), Ratio::from_integer(0)).unwrap();
    ///
    /// assert_eq!(dark_red, "4r".parse().unwrap());
    /// ```
    ///
    /// # Errors
    /// Will return `Err` if the SHT components are incompatible or impossible.
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

    /// Splits an [`SHT`] value into its struct fields.
    ///
    /// # Example
    /// ```
    /// use sht_colour::sht;
    ///
    /// let colour = "7r5bE".parse::<sht::SHT<u8>>().unwrap();
    ///
    /// let (channel_ratios, shade, tint) = colour.components();
    /// let new_colour = <sht::SHT<_>>::new(channel_ratios, shade, tint).unwrap();
    ///
    /// assert_eq!(colour, new_colour);
    /// ```
    pub fn components(&self) -> (ChannelRatios<T>, Ratio<T>, Ratio<T>) {
        let &Self {
            ref channel_ratios,
            ref shade,
            ref tint,
        } = self;
        (channel_ratios.clone(), shade.clone(), tint.clone())
    }

    /// Check whether an [`SHT`] is valid according to the criteria on
    /// <https://omaitzen.com/sht/spec/>. An `SHT` colour should have a unique
    /// canonical form under those conditions.
    ///
    /// # Errors
    /// Will return `Err` if the `SHT` is not valid. The `Err` contains a vector
    /// of all detected inconsistencies in no particular order.
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

/// Parses an [`SHT`] from a string.
///
/// See the [`Display` implementation] for the format.
///
/// # Example
/// ```
/// use sht_colour::sht::SHT;
///
/// let first_colour = "5r600000".parse::<SHT<u8>>().unwrap();
/// let second_colour = "500r6".parse::<SHT<u8>>().unwrap();
///
/// assert_eq!(first_colour, second_colour);
/// ```
///
/// [`Display` implementation]: SHT#impl-Display
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

/// Possibly rounds a base 12 number.
///
/// If `round_up`, adds 1 to the number.
/// Othewise, leaves number unchanged.
/// Number is a slice of u8 digits.
///
/// # Example
/// ```ignore
/// let arr = [1, 5, 11, 11, 11, 11];
///
/// assert_eq!(round(&arr, false), arr);
/// assert_eq!(round(&arr, true), vec![1, 6]);
/// ```
fn round(input: &[u8], round_up: bool) -> Vec<u8> {
    if round_up {
        if let Some((&last, rest)) = input.split_last() {
            let rounded_last = last.checked_add(1).unwrap_or(12);
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

/// Converts a ratio to a fixed-point base-12 string.
///
/// Output uses 'X' to represent decimal 10, and 'E' to represent decimal digit
/// 11. The output does not use '.' and does not support negative numbers.
///
/// # Example
/// ```ignore
/// use num::rational::Ratio;
///
/// assert_eq!(duodecimal(Ratio::new(11310, 20736), 2), "67");
/// ```
fn duodecimal<T>(mut input: Ratio<T>, precision: usize) -> String
where
    T: TryInto<usize> + Integer + Zero + Rem<T, Output = T> + Div<T, Output = T> + Clone,
    u8: Into<T>,
{
    let half = || Ratio::new(1.into(), 2.into());
    let digit_characters = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'X', 'E'];
    let mut digits = Vec::with_capacity(precision);
    if input >= <_>::one() {
        return "W".to_owned();
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
            Ok(n) if n < 12 => n
                .try_into()
                .expect("usize < 12 could not be converted to u8"),
            _ => 12_u8,
        };
        digits.push(next_digit);
        if input.is_zero() {
            break;
        }
    }
    // possibly round up, then convert &[u8] to digit String
    round(&digits, round_up)
        .iter()
        .map(|&c| digit_characters.get(usize::from(c)).unwrap_or(&'W'))
        .collect()
}

/// Formats the colour per the [`SHT`] format on <https://omaitzen.com/sht/spec/>:
///
/// Supports an optional `precision` parameter, which determines the maximum
/// number of digits.
///
/// # Format
///
/// > ```text
/// > [<shade>] [<primary> [<blend> <direction>] | <secondary>] [<tint>]
/// > ```
///
/// Here `<shade>`, `<blend>` and `<tint>` are numbers between 0 and 1
/// inclusive, `<primary>` and `<direction>` are primary colours, and
/// `<secondary>` is a secondary colour.
///
/// Numbers are represented using one or more base-12 digits (where `'X'` and
/// `'E'` are 10 and 11 respectively). Tint is represented by `'W'` if it is
/// equal to 12/12, i.e. the colour is pure white.
///
/// Primary colours are `'r'`, `'g'` or `'b'`, representing red, blue and green
/// respectively.
///
/// Secondary colours are `'c'`, `'y'` or `'m'`, representing cyan, yellow and
/// magenta respectively.
///
/// # Example
/// ```
/// use sht_colour::sht::SHT;
///
/// let colour = "8r6g3".parse::<SHT<u8>>().unwrap();
///
/// assert_eq!(format!("{}", colour), "8r6g3");
/// ```
impl<T> Display for SHT<T>
where
    T: TryInto<usize> + Unsigned + Integer + Clone + Display + One,
    u8: Into<T>,
{
    fn fmt(&self, formatter: &mut Formatter) -> FMTResult {
        let precision = formatter.precision().unwrap_or(2);

        let ratio_to_str = |ratio: Ratio<T>| duodecimal(ratio, precision);
        let primary_to_str = |primary| match primary {
            ColourChannel::Red => "r".to_owned(),
            ColourChannel::Green => "g".to_owned(),
            ColourChannel::Blue => "b".to_owned(),
        };
        let secondary_to_str = |secondary| match secondary {
            SecondaryColour::Cyan => "c".to_owned(),
            SecondaryColour::Yellow => "y".to_owned(),
            SecondaryColour::Magenta => "m".to_owned(),
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
            shade.map_or_else(String::new, ratio_to_str),
            primary.map_or_else(String::new, primary_to_str),
            blend.map_or_else(String::new, ratio_to_str),
            direction.map_or_else(String::new, primary_to_str),
            secondary.map_or_else(String::new, secondary_to_str),
            tint.map_or_else(String::new, ratio_to_str)
        )
    }
}

#[cfg(test)]
mod tests;

/// Contains functions for parsing [`SHT`] values and their components from
/// strings.
mod parser;
