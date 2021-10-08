//! `sht-colour` is for conversions involving SHT colour codes.
//! SHT codes are an intuitive human-readable text format for colours.
//! See <https://omaitzen.com/sht/spec/> for the specification.
//! Supports conversion to and from RGB/hex and parsing from text.
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc(test(attr(deny(warnings))))]

use num::{checked_pow, rational::Ratio, CheckedMul, Integer, One, Unsigned, Zero};

/// Support for RGB colour codes
pub mod rgb;
/// Support for SHT colour codes
pub mod sht;

#[cfg(test)]
mod lib_tests;

/// Round a ratio to a simpler approximation, in a given base.
///
/// # Arguments
/// * `ratio_on_unit_interval` - A [`Ratio<T>`] between 0 and 1 inclusive, which
///   will be rounded to a certain precision.
/// * `base` - The number base to round within.
/// * `exponent` - How many digits in that base to preserve.
/// * `negative_offset` - Usually 0. If 1, then subtract one from the
///   exponentiated base. Useful for hex codes because they are interpreted as a
///   fraction over 0xFF rather than over 0x100, meaning they have one less
///   representable value than normal.
///
/// # Errors
/// Will panic if the exponentiation overflows the integer type.
///
/// [`Ratio<T>`]: num::rational::Ratio
fn round_denominator<T>(
    ratio_on_unit_interval: Ratio<T>,
    base: T,
    exponent: usize,
    negative_offset: T,
) -> Ratio<T>
where
    T: Integer + Unsigned + CheckedMul + Clone + From<u8>,
{
    let half = Ratio::new(1.into(), 2.into());
    let new_denominator =
        checked_pow(base, exponent).expect("Overflow calculating denominator") - negative_offset;
    ((ratio_on_unit_interval * new_denominator.clone() + half).trunc()) / new_denominator
}

/// Convert a colour from [`SHT`] format to [`RGB`].
///
/// # Arguments
/// * `input` - The SHT value to convert to RGB.
/// * `precision` - How many hex digits to round the result of conversion to.
///
/// # Example
/// ```
/// use sht_colour::{rgb::RGB, sht::SHT, sht_to_rgb};
///
/// let red_rgb = "#F00".parse::<RGB<u32>>().unwrap();
/// let red_sht = "r".parse::<SHT<u32>>().unwrap();
///
/// assert_eq!(sht_to_rgb(&red_sht, 1), red_rgb);
/// ```
///
/// [`SHT`]: sht::SHT
/// [`RGB`]: rgb::RGB
pub fn sht_to_rgb<T>(input: &sht::SHT<T>, precision: usize) -> rgb::RGB<T>
where
    T: Integer + Unsigned + From<u8> + Clone + CheckedMul,
{
    let round = |ratio: Ratio<T>| round_denominator::<T>(ratio, 16.into(), precision, <_>::one());
    let (channel_ratios, shade, tint) = input.components();
    let (max, min) = (
        tint.clone() + shade * (<Ratio<_>>::one() - tint.clone()),
        tint,
    );
    let (red, green, blue) = match channel_ratios {
        sht::ChannelRatios::ThreeBrightestChannels => (min.clone(), min.clone(), min),
        sht::ChannelRatios::TwoBrightestChannels { secondary } => match secondary {
            sht::SecondaryColour::Cyan => (min, max.clone(), max),
            sht::SecondaryColour::Yellow => (max.clone(), max, min),
            sht::SecondaryColour::Magenta => (max.clone(), min, max),
        },
        sht::ChannelRatios::OneBrightestChannel {
            primary,
            direction_blend,
        } => {
            let (mut red, mut green, mut blue) = (min.clone(), min.clone(), min.clone());
            if let Some((direction, blend)) = direction_blend {
                let centremost_channel = min.clone() + blend * (max.clone() - min);
                match direction {
                    sht::ColourChannel::Red => red = centremost_channel,
                    sht::ColourChannel::Green => green = centremost_channel,
                    sht::ColourChannel::Blue => blue = centremost_channel,
                }
            };
            match primary {
                sht::ColourChannel::Red => red = max,
                sht::ColourChannel::Green => green = max,
                sht::ColourChannel::Blue => blue = max,
            };
            (red, green, blue)
        }
    };
    rgb::RGB::new(round(red), round(green), round(blue))
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
/// # Errors
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

/// Convert a colour from [`RGB`] format to [`SHT`].
///
/// # Arguments
/// * `input` - The RGB value to convert to SHT.
/// * `precision` - How many duodecimal digits to round the result of conversion
///   to.
///
/// # Example
/// ```
/// use sht_colour::{rgb::RGB, rgb_to_sht, sht::SHT};
///
/// let red_sht = "r".parse::<SHT<u32>>().unwrap();
/// let red_rgb = "#F00".parse::<RGB<u32>>().unwrap();
///
/// assert_eq!(rgb_to_sht(&red_rgb, 1), red_sht);
/// ```
///
/// # Errors
/// **Panics on overflow!**
///
/// [`SHT`]: sht::SHT
/// [`RGB`]: rgb::RGB
pub fn rgb_to_sht<T>(input: &rgb::RGB<T>, precision: usize) -> sht::SHT<T>
where
    T: Integer + Unsigned + Clone + From<u8> + CheckedMul,
{
    let round = |ratio: Ratio<T>| round_denominator::<T>(ratio, 12.into(), precision, <_>::zero());
    let (red_hex, green_hex, blue_hex) = input.components();
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
    sht::SHT::new(channel_ratios, shade, tint).expect("RGB to SHT should only create valid codes!")
}
