use super::{round_denominator, sht};
use ::num::{checked_pow, rational::Ratio, CheckedMul, Integer, One, Unsigned, Zero};
use ::std::{
    fmt::{Display, Error, Formatter, Result as FMTResult, UpperHex},
    str::FromStr,
};

/// Re-export from the `RGB` crate, representing the RGB pixel.
pub use ::rgb::RGB;

/// Represents possible errors parsing an [`HexRGB`] hex code from a string.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

/// Represents a standard RGB code in the hex format.
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
/// use ::sht_colour::{rgb::HexRGB, Ratio};
///
/// // Very bright red, some green, very little blue
/// let hex_code = "#FF8811";
///
/// // Parse colour from string
/// let parsed_colour = hex_code.parse::<HexRGB<u16>>().unwrap();
///
/// // Construct colour manually
/// let constructed_colour = HexRGB::new(
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct HexRGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    /// Inner RGB struct, which stores three colour channels, red, blue and
    /// green.
    #[doc(hidden)]
    inner: RGB<Ratio<T>>,
}

impl<T> HexRGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    /// Constructs a [`HexRGB`] value.
    ///
    /// # Arguments
    ///
    /// * `red` - The absolute brightness of the red channel.
    /// * `green` - The absolute brightness of the green channel.
    /// * `blue` - The absolute brightness of the blue channel.
    ///
    /// # Example
    /// ```
    /// use ::sht_colour::{rgb::HexRGB, Ratio};
    ///
    /// let dark_red = <HexRGB<u8>>::new(
    ///     Ratio::new(0x5, 0xF),
    ///     Ratio::new(0x0, 0xF),
    ///     Ratio::new(0x0, 0xF),
    /// );
    ///
    /// assert_eq!(dark_red, "#500".parse().unwrap());
    /// ```
    pub fn new(red: Ratio<T>, green: Ratio<T>, blue: Ratio<T>) -> HexRGB<T> {
        HexRGB {
            inner: RGB::new(red, green, blue),
        }
    }

    /// Splits a [`HexRGB`] value into its individual components, the channels
    /// red, green and blue.
    ///
    /// # Example
    /// ```
    /// use ::sht_colour::rgb;
    ///
    /// let colour = "#123456".parse::<rgb::HexRGB<u16>>().unwrap();
    ///
    /// let (red, green, blue) = colour.clone().components();
    /// let new_colour = <rgb::HexRGB<_>>::new(red, green, blue);
    ///
    /// assert_eq!(colour, new_colour);
    /// ```
    pub fn components(self) -> (Ratio<T>, Ratio<T>, Ratio<T>) {
        let Self {
            inner: RGB { r, g, b },
        } = self;
        (r, g, b)
    }

    /// Convert a colour from [`HexRGB`] format to [`SHT`].
    ///
    /// # Arguments
    /// * `precision` - How many duodecimal digits to round the result of
    ///   conversion to.
    ///
    /// # Example
    /// ```
    /// use ::sht_colour::{rgb::HexRGB, sht::SHT};
    ///
    /// let red_sht = "r".parse::<SHT<u32>>().unwrap();
    /// let red_rgb = "#F00".parse::<HexRGB<u32>>().unwrap();
    ///
    /// assert_eq!(red_rgb.to_sht(1), red_sht);
    /// ```
    ///
    /// # Panics
    /// **Panics on overflow!**
    ///
    /// [`SHT`]: sht::SHT
    pub fn to_sht(self, precision: usize) -> sht::SHT<T>
    where
        T: Integer + Unsigned + Clone + From<u8> + CheckedMul,
    {
        // Round duodecimal number to precision
        let round =
            |ratio: Ratio<T>| round_denominator::<T>(ratio, 12.into(), precision, <_>::zero());

        let (red_hex, green_hex, blue_hex) = self.components();
        let mut channels = [(red_hex, 'r'), (green_hex, 'g'), (blue_hex, 'b')];
        channels.sort();
        let [(minimum, _), (middle, mid_channel), (maximum, max_channel)] = channels;

        let tint = round(minimum.clone());
        let shade = if maximum.is_zero() {
            <num::rational::Ratio<_>>::zero()
        } else if minimum == maximum {
            <_>::one()
        } else {
            round(
                (maximum.clone() - minimum.clone())
                    / (<num::rational::Ratio<_>>::one() - minimum.clone()),
            )
        };

        let channel_ratios;
        if maximum > middle {
            let primary = char_to_primary(max_channel);

            // if `middle == minimum`, `direction_blend` set to `None`
            let direction_blend = (middle > minimum).then(|| {
                let direction = char_to_primary(mid_channel);
                let blend = (middle - minimum.clone()) / (maximum - minimum);
                (direction, round(blend))
            });
            channel_ratios = sht::ChannelRatios::OneBrightestChannel {
                primary,
                direction_blend,
            };
        } else if middle > minimum {
            let secondary = chars_to_secondary(max_channel, mid_channel);
            channel_ratios = sht::ChannelRatios::TwoBrightestChannels { secondary };
        } else {
            channel_ratios = sht::ChannelRatios::ThreeBrightestChannels;
        }
        sht::SHT::new(channel_ratios, shade, tint)
            .expect("RGB to SHT should only create valid codes!")
    }
}

impl<T> From<HexRGB<T>> for RGB<Ratio<T>>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    fn from(hex: HexRGB<T>) -> Self {
        hex.inner
    }
}

impl<T> From<RGB<Ratio<T>>> for HexRGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul,
{
    fn from(rgb: RGB<Ratio<T>>) -> Self {
        Self { inner: rgb }
    }
}

impl<T> Display for HexRGB<T>
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

impl<T> FromStr for HexRGB<T>
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
        Ok(HexRGB::new(red, green, blue))
    }
}

impl<T> Default for HexRGB<T>
where
    T: Unsigned + Integer + Clone + CheckedMul + Zero + One,
{
    fn default() -> Self {
        HexRGB {
            inner: RGB {
                r: Ratio::one(),
                g: Ratio::zero(),
                b: Ratio::zero(),
            },
        }
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

/// Return the [`ColourChannel`] corresponding to a lowercase character.
///
/// # Errors
/// Will panic if given a character other than `'r'`, `'g'` or `'b'`.
///
/// [`ColourChannel`]: sht::ColourChannel
fn char_to_primary(c: char) -> sht::ColourChannel {
    match c {
        'r' => sht::ColourChannel::Red,
        'g' => sht::ColourChannel::Green,
        'b' => sht::ColourChannel::Blue,
        _ => panic!("Invalid primary colour! {}", c),
    }
}

/// Return the [`SecondaryColour`] corresponding to a pair of lowercase
/// characters (which represent the primary colours that would add to the
/// secondary colour).
///
/// # Panics
/// Will panic if the pair does not contain exactly two distinct characters from
/// `'r'`, `'g'` and `'b'`.
///
/// [`SecondaryColour`]: sht::SecondaryColour
fn chars_to_secondary(a: char, b: char) -> sht::SecondaryColour {
    match (a, b) {
        ('g', 'b') | ('b', 'g') => sht::SecondaryColour::Cyan,
        ('r', 'g') | ('g', 'r') => sht::SecondaryColour::Yellow,
        ('r', 'b') | ('b', 'r') => sht::SecondaryColour::Magenta,
        _ => panic!("Unexpected colour channel combination! {} {}", a, b),
    }
}

#[cfg(test)]
mod tests;
