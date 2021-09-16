use num::{rational::Ratio, CheckedMul, Integer, One, Unsigned, Zero};

mod rgb;
mod sht;

#[cfg(test)]
mod lib_tests;

fn round_denominator<T>(ratio_on_unit_interval: Ratio<T>, base: T, exponent: usize) -> Ratio<T>
where
    T: Integer + Unsigned + Clone + From<u8>,
{
    let half = Ratio::new(1.into(), 2.into());
    let mut new_denominator = T::one();
    for _ in 0..exponent {
        new_denominator = new_denominator * base.clone();
    }
    let res = ((ratio_on_unit_interval * new_denominator.clone() + half).trunc()) / new_denominator;
    res
}

pub fn sht_to_rgb<N, T>(_input: sht::SHT<N>, precision: usize) -> rgb::RGB<T>
where
    N: Clone + Integer + Unsigned,
    T: Integer + Unsigned + From<N> + From<u8> + Clone + CheckedMul,
{
    todo!()
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
        ('r', 'g') | ('g', 'r') => sht::SecondaryColour::Yellow,
        ('r', 'b') | ('b', 'r') => sht::SecondaryColour::Magenta,
        ('g', 'b') | ('b', 'g') => sht::SecondaryColour::Cyan,
        _ => panic!("Unexpected colour channel combination! {} {}", a, b),
    }
}

pub fn rgb_to_sht<T>(input: rgb::RGB<T>, precision: usize) -> sht::SHT<T>
where
    T: Integer + Unsigned + Clone + From<u8> + CheckedMul,
{
    let round = |ratio: Ratio<T>| round_denominator::<T>(ratio, 12.into(), precision);
    let (red_hex, green_hex, blue_hex) = input.components();
    let mut channels = [
        (round(red_hex), 'r'),
        (round(green_hex), 'g'),
        (round(blue_hex), 'b'),
    ];
    channels.sort();
    let [(min, _), (mid, mid_channel), (max, max_channel)] = channels;
    let tint = min.clone();
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
