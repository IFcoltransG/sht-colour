//! `sht-colour` is for conversions involving SHT colour codes.
//! SHT codes are an intuitive human-readable text format for colours.
//! See <https://omaitzen.com/sht/spec/> for the specification.
//! Supports conversion to and from RGB/hex and parsing from text.

use num::{checked_pow, rational::Ratio, CheckedMul, Integer, One, Unsigned, Zero};

/// Support for RGB colour codes
pub mod rgb;
/// Support for SHT colour codes
pub mod sht;

#[cfg(test)]
mod lib_tests;

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
            if let Some(direction_blend) = direction_blend {
                let get_mid = |blend| min.clone() + blend * (max.clone() - min);
                match direction_blend {
                    (sht::ColourChannel::Red, blend) => red = get_mid(blend),
                    (sht::ColourChannel::Green, blend) => green = get_mid(blend),
                    (sht::ColourChannel::Blue, blend) => blue = get_mid(blend),
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

fn char_to_primary(c: char) -> sht::ColourChannel {
    match c {
        'r' => sht::ColourChannel::Red,
        'g' => sht::ColourChannel::Green,
        'b' => sht::ColourChannel::Blue,
        _ => panic!("Invalid primary colour! {}", c),
    }
}

fn char_to_secondary(a: char, b: char) -> sht::SecondaryColour {
    match (a, b) {
        ('g', 'b') | ('b', 'g') => sht::SecondaryColour::Cyan,
        ('r', 'g') | ('g', 'r') => sht::SecondaryColour::Yellow,
        ('r', 'b') | ('b', 'r') => sht::SecondaryColour::Magenta,
        _ => panic!("Unexpected colour channel combination! {} {}", a, b),
    }
}

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
        let secondary = char_to_secondary(max_channel, mid_channel);
        channel_ratios = sht::ChannelRatios::TwoBrightestChannels { secondary };
    } else {
        channel_ratios = sht::ChannelRatios::ThreeBrightestChannels;
    }
    sht::SHT::new(channel_ratios, shade, tint).expect("RGB to SHT should only create valid codes!")
}
