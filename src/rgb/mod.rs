use num::{checked_pow, rational::Ratio, CheckedMul, Integer, Unsigned};
use std::{
    fmt::{Display, Error, Formatter, Result as FMTResult, UpperHex},
    str::FromStr,
};

/// Represents possible errors parsing an [`RGB`] hex code from a string.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ParseHexError {
    /// The code did not have any characters.
    EmptyCode,
    /// The code did not begin with a `'#'`
    MissingOctothorpe,
    /// The code contained a number of digits that was not a multiple of three.
    /// (Transparency is not supported.)
    InvalidDigitCount,
    /// Some failure parsing digits.
    DigitParseError,
    /// The code was too large to be parsed.
    Overflow,
}

/// Represents a standard RGB code.
///
/// RGB is a common colour format that is easily interoperable with most
/// screens.
///
/// RGB colours are usually written as strings of the form `"#XXYYZZ"` called
/// hex codes, where `X`, `Y` and `Z` are hexadecimal digits for the red, green
/// and blue colour channels, respectively.
///
/// The codes can be abbreviated `#XYZ` if precision is not required.
/// Conversely, colours can be made more precise by adding digits.
///
/// # Example
/// ```
/// use num::rational::Ratio;
/// use sht_colour::rgb::RGB;
///
/// // Very bright red, some green, very little blue
/// let hex_code = "#FF8811";
///
/// // Parse colour from string
/// let parsed_colour = hex_code.parse::<RGB<u16>>().unwrap();
///
/// // Construct colour manually
/// let constructed_colour = RGB::new(
///     Ratio::new(0xFF, 0xFF),
///     Ratio::new(0x88, 0xFF),
///     Ratio::new(0x11, 0xFF),
/// );
///
/// // Both colours are the same
/// assert_eq!(constructed_colour, parsed_colour);
/// // The colour's string representation is the same as the original string
/// assert_eq!(constructed_colour.to_string(), hex_code);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    /// The brightness of the red colour channel.
    red: Ratio<T>,
    /// The brightness of the green colour channel.
    green: Ratio<T>,
    /// The brightness of the blue colour channel.
    blue: Ratio<T>,
}

impl<T> RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    /// Constructs an [`RGB`] value.
    ///
    /// # Arguments
    ///
    /// * `red` - The absolute brightness of the red channel.
    /// * `green` - The absolute brightness of the green channel.
    /// * `blue` - The absolute brightness of the blue channel.
    ///
    /// # Example
    /// ```
    /// use num::rational::Ratio;
    /// use sht_colour::rgb::RGB;
    ///
    /// let dark_red = <RGB<u8>>::new(
    ///     Ratio::new(0x5, 0xF),
    ///     Ratio::new(0x0, 0xF),
    ///     Ratio::new(0x0, 0xF),
    /// );
    ///
    /// assert_eq!(dark_red, "#500".parse().unwrap());
    /// ```
    pub fn new(red: Ratio<T>, green: Ratio<T>, blue: Ratio<T>) -> RGB<T> {
        RGB { red, green, blue }
    }

    /// Splits an [`RGB`] value into its struct fields.
    ///
    /// # Example
    /// ```
    /// use sht_colour::rgb;
    ///
    /// let colour = "#123456".parse::<rgb::RGB<u16>>().unwrap();
    ///
    /// let (red, green, blue) = colour.components();
    /// let new_colour = <rgb::RGB<_>>::new(red, green, blue);
    ///
    /// assert_eq!(colour, new_colour);
    /// ```
    pub fn components(&self) -> (Ratio<T>, Ratio<T>, Ratio<T>) {
        let &Self {
            ref red,
            ref green,
            ref blue,
        } = self;
        (red.clone(), green.clone(), blue.clone())
    }
}

impl<T> Display for RGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul + From<u8> + UpperHex,
{
    fn fmt(&self, formatter: &mut Formatter) -> FMTResult {
        let width = formatter.width().unwrap_or(2);
        let denominator = checked_pow(<T>::from(16), width).ok_or(Error)? - <T>::one();

        let from_ratio = |ratio: Ratio<T>| {
            ratio
                .checked_mul(&Ratio::from_integer(denominator.clone()))
                .ok_or(Error)
        };

        let (red, green, blue) = self.clone().components();
        write!(
            formatter,
            "#{:0width$X}{:0width$X}{:0width$X}",
            from_ratio(red)?.to_integer(),
            from_ratio(green)?.to_integer(),
            from_ratio(blue)?.to_integer(),
            width = width
        )
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

        let (red_digits, green_digits, blue_digits) = channel_split(digits)?;
        let (red, green, blue) = (
            parse_channel(red_digits)?,
            parse_channel(green_digits)?,
            parse_channel(blue_digits)?,
        );
        Ok(RGB::new(red, green, blue))
    }
}

/// Splits a string into exact thirds.
///
/// May give incorrect results if the string length is not a multiple of three.
///
/// # Errors
/// Returns `Err` if finding the second position to split at, which should
/// hopefully never happen because the original length was longer than that.
fn channel_split(s: &str) -> Result<(&str, &str, &str), ParseHexError> {
    let first = s.len() / 3;
    let second = first.checked_mul(2).ok_or(ParseHexError::Overflow)?;
    Ok((&s[..first], &s[first..second], &s[second..]))
}

/// Parses a string of hexadecimal digits into a ratio between 0 and 1
/// inclusive.
///
/// # Errors
/// Will return `Err` if a digit could not be parsed as an number, or if an
/// overflow is encountered calculating a denominator for the ratio.
fn parse_channel<T>(digits: &str) -> Result<Ratio<T>, ParseHexError>
where
    T: Unsigned + Integer + FromStr + Clone + CheckedMul,
    u8: Into<T>,
{
    Ok(<Ratio<T>>::new(
        T::from_str_radix(digits, 16).map_err(|_| ParseHexError::DigitParseError)?,
        // Equivalent to (2 ** (len(digits) * 4)) - 1
        checked_pow(
            2.into(),
            digits.len().checked_mul(4).ok_or(ParseHexError::Overflow)?,
        )
        .ok_or(ParseHexError::Overflow)?
            - T::one(),
    ))
}

#[cfg(test)]
mod tests;
