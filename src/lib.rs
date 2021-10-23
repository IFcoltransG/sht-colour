//! `sht-colour` is for conversions involving SHT colour codes.
//! SHT codes are an intuitive human-readable text format for colours.
//! See <https://omaitzen.com/sht/spec/> for the specification.
//! Supports conversion to and from RGB/hex and parsing from text.
//!
//! # Example
//! ```
//! use ::sht_colour::{
//!     rgb::{HexRGB, RGB},
//!     Ratio, SHT,
//! };
//!
//! let red_sht = "r".parse::<SHT<u8>>().unwrap();
//! let red_hex = "#F00".parse::<HexRGB<u8>>().unwrap();
//!
//! // `RGB` is the standard struct for RGB values, from the `rgb` crate.
//! let red_rgb = <RGB<Ratio<u8>>>::new(
//!     Ratio::from_integer(1),
//!     Ratio::from_integer(0),
//!     Ratio::from_integer(0),
//! );
//!
//! // Converting between SHT and HexRGB (with a precision of 1 digit).
//! assert_eq!(red_sht.to_rgb(1), red_hex);
//! assert_eq!(red_sht, red_hex.to_sht(1));
//!
//! // Converting between HexRGB and RGB.
//! assert_eq!(<RGB<Ratio<u8>>>::from(red_hex), red_rgb);
//! assert_eq!(red_hex, <HexRGB<u8>>::from(red_rgb));
//! ```
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::missing_docs_in_private_items)]

use ::num::{checked_pow, CheckedMul, Integer, Unsigned};

/// Re-export from `num` crate, represents the ratio between two numbers.
pub use ::num::rational::Ratio;
pub use sht::{ChannelRatios, ColourChannel, SecondaryColour, SHT};

/// Support for RGB colour codes in hex format.
pub mod rgb;
/// Support for SHT colour codes in SHT format.
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
/// # Panics
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
