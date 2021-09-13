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
            Ratio::new(1, 3),
            Ratio::new(1, 4),
        ),
        (
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: None,
            },
            Ratio::new(1, 2),
            Ratio::new(1, 3),
        ),
        (
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan,
            },
            Ratio::new(1, 2),
            Ratio::new(1, 3),
        ),
        (
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 2),
            Ratio::new(1, 3),
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
            SHT::<u32>::new(*channel_ratios, *shade, *tint),
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
            Ratio::new(0, 1), // error
            Ratio::new(1, 2),
        ),
        Err(vec![SHTValueError::PrimaryShadeZero])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: None
            },
            Ratio::new(1, 2),
            Ratio::new(1, 1), // error
        ),
        Err(vec![SHTValueError::PrimaryTintOne])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            },
            Ratio::new(0, 1), // error
            Ratio::new(1, 2),
        ),
        Err(vec![SHTValueError::SecondaryShadeZero])
    );
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            },
            Ratio::new(1, 2),
            Ratio::new(1, 1), // error
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
            Ratio::new(1, 2),
        ),
        Err(vec![SHTValueError::DirectionEqualsPrimary])
    );
    for sht_code in [
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
            },
            Ratio::new(1, 2),
            Ratio::new(2, 1), // error
        ),
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
            },
            Ratio::new(2, 1), // error
            Ratio::new(1, 2),
        ),
        SHT::<u32>::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 2),
            Ratio::new(2, 1), // error
        ),
        SHT::<u32>::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(2, 1), // error
            Ratio::new(1, 2),
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
            Ratio::new(1, 2),
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
            Ratio::new(1, 2),
        ),
        Err(vec![SHTValueError::BlendOne])
    );
}

#[test]
fn parse_success() {
    use super::{ChannelRatios, ColourChannel, ParsePropertyError, SecondaryColour, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "W".parse::<SHT<u8>>(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 1)
        )
        .map_err(ParsePropertyError::ValueErrors)
    );
    assert_eq!(
        "8r6g3...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
            },
            Ratio::new(2, 3),
            Ratio::new(1, 4)
        )
        .ok()
    );
    assert_eq!(
        "r...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            },
            Ratio::new(1, 1),
            Ratio::new(0, 1),
        )
        .ok()
    );
    assert_eq!(
        "8r...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            },
            Ratio::new(2, 3),
            Ratio::new(0, 1),
        )
        .ok()
    );
    assert_eq!(
        "r3...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            },
            Ratio::new(1, 1),
            Ratio::new(1, 4),
        )
        .ok()
    );
    assert_eq!(
        "6r3...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            },
            Ratio::new(1, 2),
            Ratio::new(1, 4),
        )
        .ok()
    );
    assert_eq!(
        "r6g...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
            },
            Ratio::new(1, 1),
            Ratio::new(0, 1),
        )
        .ok()
    );
    assert_eq!(
        "8r6g...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
            },
            Ratio::new(2, 3),
            Ratio::new(0, 1),
        )
        .ok()
    );
    assert_eq!(
        "8r6g3...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
            },
            Ratio::new(2, 3),
            Ratio::new(1, 4),
        )
        .ok()
    );
    assert_eq!(
        "8y3...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Yellow
            },
            Ratio::new(2, 3),
            Ratio::new(1, 4),
        )
        .ok()
    );
    assert_eq!(
        "6...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 2),
        )
        .ok()
    );
    assert_eq!(
        "0...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(0, 1),
            Ratio::new(0, 1),
        )
        .ok()
    );
    assert_eq!(
        "W...".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 1),
        )
        .ok()
    );
}

#[test]
fn parse_failure() {
    use super::{ParsePropertyError, SHT};
    use nom::{error::Error, error::ErrorKind};
    assert_eq!(
        "".parse::<SHT<u8>>(),
        Err(ParsePropertyError::ParseFailure(Error::new(
            "".to_owned(),
            ErrorKind::Tag
        )))
    );
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
    use super::parser::quantity;
    use num::rational::Ratio;
    assert_eq!(quantity("1C"), Ok(("C", Ratio::new(1u32, 12))));
    assert_eq!(quantity("11C"), Ok(("C", Ratio::new(13u32, 144))));
    assert_eq!(quantity("EEEC"), Ok(("C", Ratio::new(1727u32, 1728))));
    assert_eq!(quantity("EEC"), Ok(("C", Ratio::new(143u32, 144))));
    // 144 is the largest power of 12 that fits in a u8
    assert_eq!(quantity("EEEEEEC"), Ok(("C", Ratio::new(143u8, 144))));
    // 20736 is the largest power of 12 that fits in a u16
    assert_eq!(quantity("EEEEEEC"), Ok(("C", Ratio::new(20735u16, 20736))));
    // u32 supports all 6 digits of precision
    assert_eq!(
        quantity("EEEEEEC"),
        Ok(("C", Ratio::new(2985983u32, 2985984)))
    );
}

#[test]
fn parse_quantity_error() {
    use super::parser::quantity;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        quantity::<u8>("C"),
        Err(Err::Error(Error::new("C", ErrorKind::Many1)))
    );
}

#[test]
fn parse_direction_blend_success() {
    use super::{parser::direction_blend, ColourChannel};
    use num::rational::Ratio;
    assert_eq!(
        direction_blend("34EXRC"),
        Ok(("C", (ColourChannel::Red, Ratio::new(5902u32, 20736))))
    );
    assert_eq!(
        direction_blend("3GC"),
        Ok(("C", (ColourChannel::Green, Ratio::new(3u32, 12))))
    );
}

#[test]
fn parse_direction_blend_failure() {
    use super::parser::direction_blend;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        direction_blend::<u8>("..."),
        Err(Err::Error(Error::new("...", ErrorKind::Many1)))
    );
    assert_eq!(
        direction_blend::<u8>("R1..."),
        Err(Err::Error(Error::new("R1...", ErrorKind::Many1)))
    );
    assert_eq!(
        direction_blend::<u8>("R..."),
        Err(Err::Error(Error::new("R...", ErrorKind::Many1)))
    );
    assert_eq!(
        direction_blend::<u8>("1..."),
        Err(Err::Error(Error::new("...", ErrorKind::Tag)))
    );
}

#[test]
fn parse_channel_ratios() {
    use super::{parser::channel_ratios, ChannelRatios, ColourChannel, SecondaryColour};
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    use num::rational::Ratio;
    assert_eq!(
        channel_ratios::<u8>("R..."),
        Ok((
            "...",
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            }
        ))
    );
    assert_eq!(
        channel_ratios::<u16>("R123G..."),
        Ok((
            "...",
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: Some((ColourChannel::Green, Ratio::new(171, 1728)))
            }
        ))
    );
    assert_eq!(
        channel_ratios::<u16>("R123..."),
        Ok((
            "123...",
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Red,
                direction_blend: None
            }
        ))
    );
    assert_eq!(
        channel_ratios::<u16>("G..."),
        Ok((
            "...",
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Green,
                direction_blend: None
            }
        ))
    );
    assert_eq!(
        channel_ratios::<u16>("C..."),
        Ok((
            "...",
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            }
        ))
    );
    assert_eq!(
        channel_ratios::<u8>("..."),
        Err(Err::Error(Error::new("...", ErrorKind::Tag)))
    );
    assert_eq!(
        channel_ratios::<u16>("123R..."),
        Err(Err::Error(Error::new("123R...", ErrorKind::Tag)))
    );
}

#[test]
fn parse_sht_data() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel, SecondaryColour};
    use num::rational::Ratio;
    assert_eq!(
        sht_data::<u8>("8r6g3..."),
        Ok((
            "...",
            (
                Some(Ratio::new(2, 3)),
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
                },
                Some(Ratio::new(1, 4))
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("r..."),
        Ok((
            "...",
            (
                None,
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: None
                },
                None,
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("8r..."),
        Ok((
            "...",
            (
                Some(Ratio::new(2, 3)),
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: None
                },
                None,
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("r3..."),
        Ok((
            "...",
            (
                None,
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: None
                },
                Some(Ratio::new(1, 4)),
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("6r3..."),
        Ok((
            "...",
            (
                Some(Ratio::new(1, 2)),
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: None
                },
                Some(Ratio::new(1, 4)),
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("r6g..."),
        Ok((
            "...",
            (
                None,
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
                },
                None,
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("8r6g..."),
        Ok((
            "...",
            (
                Some(Ratio::new(2, 3)),
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
                },
                None,
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("8r6g3..."),
        Ok((
            "...",
            (
                Some(Ratio::new(2, 3)),
                ChannelRatios::OneBrightestChannel {
                    primary: ColourChannel::Red,
                    direction_blend: Some((ColourChannel::Green, Ratio::new(1, 2)))
                },
                Some(Ratio::new(1, 4)),
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("8y3..."),
        Ok((
            "...",
            (
                Some(Ratio::new(2, 3)),
                ChannelRatios::TwoBrightestChannels {
                    secondary: SecondaryColour::Yellow
                },
                Some(Ratio::new(1, 4)),
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("6..."),
        Ok((
            "...",
            (
                None,
                ChannelRatios::ThreeBrightestChannels,
                Some(Ratio::new(1, 2)),
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("0..."),
        Ok((
            "...",
            (
                Some(Ratio::new(0, 1)),
                ChannelRatios::ThreeBrightestChannels,
                None,
            )
        ))
    );
    assert_eq!(
        sht_data::<u8>("W..."),
        Ok((
            "...",
            (
                None,
                ChannelRatios::ThreeBrightestChannels,
                Some(Ratio::new(1, 1)),
            )
        ))
    );
}

#[test]
fn sht_parse_success() {
    todo!()
}

#[test]
fn sht_parse_failure() {
    todo!()
}

#[test]
fn case_works() {
    unimplemented!();
}
