#[test]
fn value_success() {
    use super::{ChannelRatios, ColourChannel, SecondaryColour, SHT};
    use num::rational::Ratio;
    for (channel_ratios, tint, shade) in [
        (
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
        (
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: None,
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
        (
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan,
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
        (
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
        (
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 1),
        ),
        (
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(0, 1),
            Ratio::new(0, 1),
        ),
    ]
    .iter()
    {
        assert_eq!(
            SHT::<u32>::new(*channel_ratios, *tint, *shade),
            Ok(SHT::<u32> {
                channel_ratios: *channel_ratios,
                tint: *tint,
                shade: *shade
            })
        )
    }
}

#[test]
fn value_failure() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SecondaryColour, SHT};
    use num::rational::Ratio;
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: None
            },
            Ratio::new(1, 2),
            Ratio::new(0, 1) // error
        ),
        Err(vec![SHTValueError::PrimaryShadeZero])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: None
            },
            Ratio::new(1, 1), // error
            Ratio::new(1, 2)
        ),
        Err(vec![SHTValueError::PrimaryTintOne])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            },
            Ratio::new(1, 2),
            Ratio::new(0, 1) // error
        ),
        Err(vec![SHTValueError::SecondaryShadeZero])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            },
            Ratio::new(1, 1), // error
            Ratio::new(1, 2)
        ),
        Err(vec![SHTValueError::SecondaryTintOne])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((
                    ColourChannel::Blue, //error
                    Ratio::new(1u32, 2)
                ))
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2)
        ),
        Err(vec![SHTValueError::DirectionEqualsPrimary])
    );
    for sht_code in [
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
            },
            Ratio::new(2, 1), // error
            Ratio::new(1, 2),
        ),
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
            },
            Ratio::new(1, 2),
            Ratio::new(2, 1), // error
        ),
        SHT::<u32>::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(2, 1), // error
            Ratio::new(1, 2),
        ),
        SHT::<u32>::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 2),
            Ratio::new(2, 1), // error
        ),
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(2, 1))), // error
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
    ]
    .iter()
    {
        assert_eq!(sht_code, &Err(vec![SHTValueError::ValueOutOfBounds]));
    }
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(0, 1))) // error
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2)
        ),
        Err(vec![SHTValueError::BlendZero])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 1))) // error
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2)
        ),
        Err(vec![SHTValueError::BlendOne])
    );
}

#[test]
fn parse_success() {
    use super::{ChannelRatios, ParsePropertyError, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "W".parse::<SHT<u8>>(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(0, 1),
            Ratio::new(1, 1)
        )
        .map_err(ParsePropertyError::ValueErrors)
    );
    todo!()
}

#[test]
fn parse_failure() {
    use super::{ParsePropertyError, SHT};
    assert_eq!("".parse::<SHT<u8>>(), Err(ParsePropertyError::EmptyCode));
    todo!()
}

#[test]
fn parse_colours() {
    use super::{
        parser::{primary_colour, secondary_colour},
        ColourChannel, SecondaryColour,
    };
    assert_eq!(secondary_colour("cc"), Ok(("c", SecondaryColour::Cyan)));
    assert_eq!(secondary_colour("yc"), Ok(("c", SecondaryColour::Yellow)));
    assert_eq!(secondary_colour("mc"), Ok(("c", SecondaryColour::Magenta)));
    assert_eq!(primary_colour("rr"), Ok(("r", ColourChannel::Red)));
    assert_eq!(primary_colour("gr"), Ok(("r", ColourChannel::Green)));
    assert_eq!(primary_colour("br"), Ok(("r", ColourChannel::Blue)))
}

#[test]
fn parse_digits() {
    use super::parser::{duodecimal_digit, number_from_digit};
    let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'X', 'E'];
    for i in 1u8..=255 {
        // iterate through ASCII
        // ensure the matches exactly correspond to digits
        let c = i as char;
        assert_eq!(
            duodecimal_digit(&c.to_string()).is_ok(),
            digits.contains(&c)
        );
        assert_eq!(
            number_from_digit::<u8>(&c.to_string()).is_ok(),
            digits.contains(&c)
        )
    }
    for (i, c) in digits.iter().enumerate() {
        // iterate through 0..=12, checking the digit is right
        assert_eq!(number_from_digit(&c.to_string()), Ok(("", i)));
        assert_eq!(
            number_from_digit(&c.to_string().repeat(10)),
            Ok((c.to_string().repeat(9).as_str(), i))
        )
    }
}

#[test]
fn parse_quantity_success() {
    use super::parser::quantity_with_precision;
    use num::rational::Ratio;
    assert_eq!(
        quantity_with_precision(3)("1C"),
        Ok(("C", Ratio::new(1u32, 12)))
    );
    assert_eq!(
        quantity_with_precision(3)("11C"),
        Ok(("C", Ratio::new(13u32, 144)))
    );
    assert_eq!(
        quantity_with_precision(3)("EEEC"),
        Ok(("C", Ratio::new(1727u32, 1728)))
    );
    assert_eq!(
        quantity_with_precision(3)("EEC"),
        Ok(("C", Ratio::new(143u32, 144)))
    );
    assert_eq!(
        quantity_with_precision(1)("EEC"),
        Ok(("C", Ratio::new(11u32, 12)))
    );
}

#[test]
fn parse_quantity_error() {
    use super::parser::{quantity_with_precision, RatioParseError};
    use nom::{Err, error::ErrorKind};
    assert_eq!(
        quantity_with_precision::<u8>(1)("C"),
        Err(nom::Err::Error(RatioParseError::Nom("C", ErrorKind::Many1)))
    );
}

#[test]
fn parse_direction_blend() {
    use super::{parser::direction_blend, ColourChannel};
    use num::rational::Ratio;
    assert_eq!(
        direction_blend(4)("34EX5RC"),
        Ok(("C", (ColourChannel::Red, Ratio::new(5902u32, 20736))))
    );
    unimplemented!();
}

#[test]
fn parse_channel_ratios() {
    unimplemented!();
}

#[test]
fn parse_sht_data() {
    unimplemented!();
}
