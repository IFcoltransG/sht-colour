//! `sht-colour` is for conversions involving SHT colour codes.
//! SHT codes are an intuitive human-readable text format for colours.
//! See <https://omaitzen.com/sht/spec/> for the specification.
//! Supports conversion to and from RGB/hex and parsing from text.
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc(test(attr(deny(warnings))))]

use num::{checked_pow, rational::Ratio, CheckedMul, Integer, Unsigned};

/// Support for RGB colour codes in hex format
pub mod rgb;
/// Support for SHT colour codes in SHT format
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
