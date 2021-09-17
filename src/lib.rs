use num::{rational::Ratio, CheckedMul, Integer, One, Unsigned, Zero};
use std::fmt::Debug;

mod rgb;
mod sht;

#[cfg(test)]
mod lib_tests;

fn round_denominator<T>(
    ratio_on_unit_interval: Ratio<T>,
    base: T,
    exponent: usize,
    negative_offset: T,
) -> Ratio<T>
where
    T: Integer + Unsigned + Clone + From<u8>,
{
    let half = Ratio::new(1.into(), 2.into());
    let mut new_denominator = T::one();
    for _ in 0..exponent {
        new_denominator = new_denominator * base.clone();
    }
    new_denominator = new_denominator - negative_offset;
    let res = ((ratio_on_unit_interval * new_denominator.clone() + half).trunc()) / new_denominator;
    res
}

pub fn sht_to_rgb<T>(input: sht::SHT<T>, precision: usize) -> rgb::RGB<T>
where
    T: Integer + Unsigned + From<u8> + Clone + CheckedMul,
    T: Debug,
{
    let round = |ratio: Ratio<T>| round_denominator::<T>(ratio, 16.into(), precision, <_>::one());
    let (channel_ratios, shade, tint) = input.components();
    let (max, min) = (
        tint.clone() + shade * (<Ratio<_>>::one() - tint.clone()),
        tint,
    );
    eprintln!("MAX: {:?}", max);
    eprintln!("ROUNDED MAX: {:?}", round(max.clone()));
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

pub fn rgb_to_sht<T>(input: rgb::RGB<T>, precision: usize) -> sht::SHT<T>
where
    T: Integer + Unsigned + Clone + From<u8> + CheckedMul,
{
    let round = |ratio: Ratio<T>| round_denominator::<T>(ratio, 12.into(), precision, <_>::zero());
    let (red_hex, green_hex, blue_hex) = input.components();
    let mut channels = [(red_hex, 'r'), (green_hex, 'g'), (blue_hex, 'b')];
    channels.sort();
    let [(min, _), (mid, mid_channel), (max, max_channel)] = channels;
    let tint = round(min.clone());
    let shade = if max.is_zero() {
        <num::rational::Ratio<_>>::zero()
    } else if min.clone() != max.clone() {
        round((max.clone() - min.clone()) / (<num::rational::Ratio<_>>::one() - min.clone()))
    } else {
        <_>::one()
    };
    let channel_ratios;
    if max.clone() > mid {
        let primary = char_to_primary(max_channel);

        let direction_blend = if mid > min.clone() {
            let direction = char_to_primary(mid_channel);
            let blend = (mid - min.clone()) / (max - min);
            Some((direction, round(blend)))
        } else {
            None
        };
        channel_ratios = sht::ChannelRatios::OneBrightestChannel {
            primary,
            direction_blend,
        };
    } else if mid > min {
        let secondary = char_to_secondary(max_channel, mid_channel);
        channel_ratios = sht::ChannelRatios::TwoBrightestChannels { secondary };
    } else {
        channel_ratios = sht::ChannelRatios::ThreeBrightestChannels;
    }
    sht::SHT::new(channel_ratios, shade, tint).expect("RGB to SHT should only create valid codes!")
}
