#[test]
fn value_success_one_brightest_channel() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    for (channel_ratios, tint, shade) in &[
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
    ] {
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
fn value_success_two_brightest_channels() {
    use super::{ChannelRatios, SecondaryColour, SHT};
    use num::rational::Ratio;
    let (channel_ratios, tint, shade) = (
        ChannelRatios::TwoBrightestChannels {
            secondary: SecondaryColour::Cyan,
        },
        Ratio::new(1, 2),
        Ratio::new(1, 3),
    );
    {
        assert_eq!(
            SHT::<u32>::new(channel_ratios, shade, tint),
            Ok(SHT::<u32> {
                channel_ratios,
                tint,
                shade
            })
        )
    }
}

#[test]
fn value_success_three_brightest_channels() {
    use super::{ChannelRatios, SHT};
    use num::rational::Ratio;
    for (channel_ratios, tint, shade) in &[
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
    ] {
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
fn value_failure_primary_shade_zero() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
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
}

#[test]
fn value_failure_primary_tint_one() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
    use num::rational::Ratio;
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
}

#[test]
fn value_failure_secondary_shade_zero() {
    use super::{ChannelRatios, SHTValueError, SecondaryColour, SHT};
    use num::rational::Ratio;
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
}

#[test]
fn value_failure_secondary_tint_one() {
    use super::{ChannelRatios, SHTValueError, SecondaryColour, SHT};
    use num::rational::Ratio;
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
}

#[test]
fn value_failure_direction_equals_primary() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
    use num::rational::Ratio;
    assert_eq!(
        SHT::<u32>::new(
            ChannelRatios::OneBrightestChannel {
                primary: ColourChannel::Blue,
                direction_blend: Some((
                    ColourChannel::Blue, // error
                    Ratio::new(1_u32, 2)
                ))
            },
            Ratio::new(1, 2),
            Ratio::new(1, 2),
        ),
        Err(vec![SHTValueError::DirectionEqualsPrimary])
    );
}

#[test]
fn value_failure_out_of_bounds() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
    use num::rational::Ratio;
    for sht_code in &[
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
    ] {
        assert_eq!(sht_code, &Err(vec![SHTValueError::ValueOutOfBounds]));
    }
}

#[test]
fn value_failure_blend_zero() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
    use num::rational::Ratio;
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
}

#[test]
fn value_failure_blend_one() {
    use super::{ChannelRatios, ColourChannel, SHTValueError, SHT};
    use num::rational::Ratio;
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
fn parse_success_shade_blend_tint() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "8r6g3".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_only_primary() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "r".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_shade() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "8r".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_tint() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "r3".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_shade_tint() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "6r3".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_blend() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "r6g".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_shade_blend() {
    use super::{ChannelRatios, ColourChannel, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "8r6g".parse::<SHT<u8>>().ok(),
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
}

#[test]
fn parse_success_shade_secondary_tint() {
    use super::{ChannelRatios, SecondaryColour, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "8y3".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Yellow
            },
            Ratio::new(2, 3),
            Ratio::new(1, 4),
        )
        .ok()
    );
}

#[test]
fn parse_success_shade_only() {
    use super::{ChannelRatios, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "6".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 2),
        )
        .ok()
    );
}

#[test]
fn parse_success_zero_shade_only() {
    use super::{ChannelRatios, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "0".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(0, 1),
            Ratio::new(0, 1),
        )
        .ok()
    );
}

#[test]
fn parse_success_tint_only() {
    use super::{ChannelRatios, SHT};
    use num::rational::Ratio;
    assert_eq!(
        "W".parse::<SHT<u8>>().ok(),
        SHT::new(
            ChannelRatios::ThreeBrightestChannels,
            Ratio::new(1, 1),
            Ratio::new(1, 1),
        )
        .ok()
    );
}

#[test]
fn parse_failure_empty_string() {
    use super::{ParsePropertyError, SHT};
    use nom::error::{Error, ErrorKind};
    assert_eq!(
        "".parse::<SHT<u8>>(),
        Err(ParsePropertyError::ParseFailure(Error::new(
            "".to_owned(),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn parse_failure_leftover_symbols() {
    use super::{ParsePropertyError, SHT};
    use nom::error::{Error, ErrorKind};
    assert_eq!(
        "...".parse::<SHT<u8>>(),
        Err(ParsePropertyError::ParseFailure(Error::new(
            "...".to_owned(),
            ErrorKind::Tag
        )))
    );
}

#[test]
#[allow(non_snake_case)]
fn parse_failure_extra_W() {
    use super::{ParsePropertyError, SHT};
    let leftover = |s: &str| Err(ParsePropertyError::InputRemaining(s.to_string()));
    assert_eq!("8r6g3W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("rW".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("8rW".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("r3W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("6r3W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("r6gW".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("8r6gW".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("8r6g3W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("8y3W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("6W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("0W".parse::<SHT<u8>>(), leftover("W"));
    assert_eq!("WW".parse::<SHT<u8>>(), leftover("W"));
}

#[test]
fn parse_failure_extra_r() {
    use super::{ParsePropertyError, SHT};
    let leftover = |s: &str| Err(ParsePropertyError::InputRemaining(s.to_string()));
    assert_eq!("8r6g3r".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("rr".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("8rr".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("r6gr".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("8r6gr".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("8r6g3r".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("8y3r".parse::<SHT<u8>>(), leftover("r"));
    assert_eq!("Wr".parse::<SHT<u8>>(), leftover("r"));
}

#[test]
fn parse_failure_extra_c() {
    use super::{ParsePropertyError, SHT};
    let leftover = |s: &str| Err(ParsePropertyError::InputRemaining(s.to_string()));
    assert_eq!("8r6g3c".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("rc".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("8rc".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("r3c".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("6r3c".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("r6gc".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("8r6gc".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("8r6g3c".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("8y3c".parse::<SHT<u8>>(), leftover("c"));
    assert_eq!("Wc".parse::<SHT<u8>>(), leftover("c"));
}

#[test]
fn parse_failure_extra_0() {
    use super::{ParsePropertyError, SHT};
    let leftover = |s: &str| Err(ParsePropertyError::InputRemaining(s.to_string()));
    assert_eq!("r0".parse::<SHT<u8>>(), leftover("0"));
    assert_eq!("8r0".parse::<SHT<u8>>(), leftover("0"));
    assert_eq!("r6g0".parse::<SHT<u8>>(), leftover("0"));
    assert_eq!("8r6g0".parse::<SHT<u8>>(), leftover("0"));
    assert_eq!("W0".parse::<SHT<u8>>(), leftover("0"));
}

#[test]
fn parse_failure_extra_1() {
    use super::{ParsePropertyError, SHT};
    assert_eq!(
        "W1".parse::<SHT<u8>>(),
        Err(ParsePropertyError::InputRemaining("1".to_string()))
    );
}

#[test]
fn parse_primary_colours() {
    use super::{parser::primary_colour, ColourChannel};
    assert_eq!(primary_colour("rr"), Ok(("r", ColourChannel::Red)));
    assert_eq!(primary_colour("gr"), Ok(("r", ColourChannel::Green)));
    assert_eq!(primary_colour("br"), Ok(("r", ColourChannel::Blue)))
}

#[test]
fn parse_secondary_colours() {
    use super::{parser::secondary_colour, SecondaryColour};
    assert_eq!(secondary_colour("cc"), Ok(("c", SecondaryColour::Cyan)));
    assert_eq!(secondary_colour("yc"), Ok(("c", SecondaryColour::Yellow)));
    assert_eq!(secondary_colour("mc"), Ok(("c", SecondaryColour::Magenta)));
}

#[test]
fn parse_only_digits() {
    use super::parser::{duodecimal_digit, number_from_digit};
    let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'X', 'E'];
    for i in 0_u8..=255 {
        // iterate through ASCII
        // ensure the matches exactly correspond to digits
        let c = i as char;
        assert_eq!(
            duodecimal_digit(&c.to_string()).is_ok(),
            digits.contains(&c.to_uppercase().next().unwrap())
        );
        assert_eq!(
            number_from_digit::<u8>(&c.to_string()).is_ok(),
            digits.contains(&c)
        )
    }
}

#[test]
fn parse_all_digits() {
    use super::parser::number_from_digit;
    let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'X', 'E'];
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
fn parse_quantity_success_u32_unrounded() {
    use super::parser::quantity;
    use num::rational::Ratio;
    assert_eq!(quantity("1c"), Ok(("c", Ratio::new(1_u32, 12))));
    assert_eq!(quantity("11c"), Ok(("c", Ratio::new(13_u32, 144))));
    assert_eq!(quantity("EEEc"), Ok(("c", Ratio::new(1727_u32, 1728))));
    assert_eq!(quantity("EEc"), Ok(("c", Ratio::new(143_u32, 144))));
}

#[test]
fn parse_quantity_success_u8_upper_bounds() {
    use super::parser::quantity;
    use num::rational::Ratio;
    // 144 is the largest power of 12 that fits in a u8
    assert_eq!(quantity("EE0Ec"), Ok(("c", Ratio::new(143_u8, 144))));
    // round up
    assert_eq!(quantity("EEEEc"), Ok(("c", Ratio::new(1_u8, 1))));
}

#[test]
fn parse_quantity_success_u16() {
    use super::parser::quantity;
    use num::rational::Ratio;
    // 20736 is the largest power of 12 that fits in a u16
    assert_eq!(quantity("EEEE0Ec"), Ok(("c", Ratio::new(20735_u16, 20736))));
    assert_eq!(quantity("555555c"), Ok(("c", Ratio::new(9425_u16, 20736))));
    // however, can get another sixth or quarter of precision:
    assert_eq!(quantity("555565c"), Ok(("c", Ratio::new(18851_u16, 41472))));
    // quarter:
    assert_eq!(quantity("555585c"), Ok(("c", Ratio::new(28277_u16, 62208))));
    assert_eq!(quantity("EEEE7c"), Ok(("c", Ratio::new(1_u16, 1))));
    assert_eq!(quantity("EEEE5c"), Ok(("c", Ratio::new(20735_u16, 20736))));
}

#[test]
fn parse_quantity_success_u32_precision() {
    use super::parser::quantity;
    use num::rational::Ratio;
    // u32 supports all 6 digits of precision
    assert_eq!(
        quantity("EEEEEEc"),
        Ok(("c", Ratio::new(2_985_983_u32, 2_985_984)))
    );
}

#[test]
fn parse_quantity_success_u8_two_thirds() {
    use super::parser::quantity;
    use num::rational::Ratio;
    // round up to 67/100 in base 12
    assert_eq!(quantity("666c"), Ok(("c", Ratio::new(79_u8, 144))))
}

#[test]
fn parse_quantity_case_error() {
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
        Ok(("C", (ColourChannel::Red, Ratio::new(5902_u32, 20736))))
    );
    assert_eq!(
        direction_blend("3GC"),
        Ok(("C", (ColourChannel::Green, Ratio::new(3_u32, 12))))
    );
}

#[test]
fn parse_direction_blend_failure_empty() {
    use super::parser::direction_blend;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        direction_blend::<u8>("..."),
        Err(Err::Error(Error::new("...", ErrorKind::Many1)))
    );
}

#[test]
fn parse_direction_blend_failure_wrong_order_direction_blend() {
    use super::parser::direction_blend;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        direction_blend::<u8>("r1..."),
        Err(Err::Error(Error::new("r1...", ErrorKind::Many1)))
    );
}

#[test]
fn parse_direction_blend_failure_direction_no_blend() {
    use super::parser::direction_blend;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        direction_blend::<u8>("r..."),
        Err(Err::Error(Error::new("r...", ErrorKind::Many1)))
    );
}

#[test]
fn parse_direction_blend_failure_blend_no_direction() {
    use super::parser::direction_blend;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        direction_blend::<u8>("1..."),
        Err(Err::Error(Error::new("...", ErrorKind::Tag)))
    );
}

#[test]
fn parse_channel_ratios_uppercase_primary_u8() {
    use super::{parser::channel_ratios, ChannelRatios, ColourChannel};
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
}

#[test]
fn parse_channel_ratios_uppercase_direction_blend() {
    use super::{parser::channel_ratios, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_channel_ratios_uppercase_tint() {
    use super::{parser::channel_ratios, ChannelRatios, ColourChannel};
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
}

#[test]
fn parse_channel_ratios_uppercase_primary_u16() {
    use super::{parser::channel_ratios, ChannelRatios, ColourChannel};
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
}

#[test]
fn parse_channel_ratios_uppercase_secondary() {
    use super::{parser::channel_ratios, ChannelRatios, SecondaryColour};
    assert_eq!(
        channel_ratios::<u16>("C..."),
        Ok((
            "...",
            ChannelRatios::TwoBrightestChannels {
                secondary: SecondaryColour::Cyan
            }
        ))
    );
}

#[test]
fn parse_channel_ratios_empty() {
    use super::parser::channel_ratios;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        channel_ratios::<u8>("..."),
        Err(Err::Error(Error::new("...", ErrorKind::Tag)))
    );
}

#[test]
fn parse_channel_ratios_uppercase_shade_primary() {
    use super::parser::channel_ratios;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };
    assert_eq!(
        channel_ratios::<u16>("123R..."),
        Err(Err::Error(Error::new("123R...", ErrorKind::Tag)))
    );
}

#[test]
fn parse_sht_data_shade_blend_tint() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
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
}

#[test]
fn parse_sht_data_primary_only() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
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
}

#[test]
fn parse_sht_data_shade() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_tint() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_shade_tint() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_blend() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_shade_blend() {
    use super::{parser::sht_data, ChannelRatios, ColourChannel};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_shade_secondary_tint() {
    use super::{parser::sht_data, ChannelRatios, SecondaryColour};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_shade_only() {
    use super::{parser::sht_data, ChannelRatios};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_zero_shade_only() {
    use super::{parser::sht_data, ChannelRatios};
    use num::rational::Ratio;
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
}

#[test]
fn parse_sht_data_tint_only() {
    use super::{parser::sht_data, ChannelRatios};
    use num::rational::Ratio;
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
fn display_precision_4() {
    use super::SHT;
    assert_eq!(&format!("{:.4}", "0".parse::<SHT<u8>>().unwrap()), "0");
    assert_eq!(&format!("{:.4}", "6".parse::<SHT<u8>>().unwrap()), "6");
    assert_eq!(&format!("{:.4}", "6666".parse::<SHT<u8>>().unwrap()), "67");
    assert_eq!(
        &format!("{:.4}", "6666".parse::<SHT<u32>>().unwrap()),
        "6666"
    );
}

#[test]
fn display_precision_5_u32() {
    use super::SHT;
    assert_eq!(
        &format!("{:.5}", "123456".parse::<SHT<u32>>().unwrap()),
        "12346"
    );
}

#[test]
fn display_precision_1() {
    use super::SHT;
    assert_eq!(&format!("{:.1}", "r".parse::<SHT<u8>>().unwrap()), "r");
    assert_eq!(&format!("{:.1}", "8r".parse::<SHT<u8>>().unwrap()), "8r");
    assert_eq!(&format!("{:.1}", "r3".parse::<SHT<u8>>().unwrap()), "r3");
    assert_eq!(&format!("{:.1}", "8r3".parse::<SHT<u8>>().unwrap()), "8r3");
    assert_eq!(&format!("{:.1}", "r6g".parse::<SHT<u8>>().unwrap()), "r6g");
    assert_eq!(
        &format!("{:.1}", "8r6g".parse::<SHT<u8>>().unwrap()),
        "8r6g"
    );
    assert_eq!(
        &format!("{:.1}", "8r6g3".parse::<SHT<u8>>().unwrap()),
        "8r6g3"
    );
    assert_eq!(&format!("{:.1}", "8y3".parse::<SHT<u8>>().unwrap()), "8y3");
    assert_eq!(&format!("{:.1}", "6".parse::<SHT<u8>>().unwrap()), "6");
    assert_eq!(&format!("{:.1}", "0".parse::<SHT<u8>>().unwrap()), "0");
    assert_eq!(&format!("{:.1}", "W".parse::<SHT<u8>>().unwrap()), "W");
}

#[test]
fn display_no_precision() {
    use super::SHT;
    assert_eq!(&format!("{}", "1234".parse::<SHT<u8>>().unwrap()), "12");
}

#[test]
fn display_precision_2_u16() {
    use super::SHT;
    assert_eq!(&format!("{:.2}", "EEE".parse::<SHT<u16>>().unwrap()), "W");
}

#[test]
fn display_no_precision_upper_bound() {
    use super::SHT;
    assert_eq!(&format!("{}", "EEE".parse::<SHT<u8>>().unwrap()), "W");
}

#[test]
fn duodecimal_zero() {
    use super::duodecimal;
    use num::rational::Ratio;
    assert_eq!(duodecimal(Ratio::new(0, 1), 4), "0");
}

#[test]
fn duodecimal_half() {
    use super::duodecimal;
    use num::rational::Ratio;
    assert_eq!(duodecimal(Ratio::new(6, 12), 4), "6");
}

#[test]
fn duodecimal_two_thirds() {
    use super::duodecimal;
    use num::rational::Ratio;
    assert_eq!(duodecimal(Ratio::new(11310, 20736), 2), "67"); // 6666 / 10000 in base 12
    assert_eq!(duodecimal(Ratio::new(11310, 20736), 4), "6666"); // same, different prec
}

#[test]
fn duodecimal_high_precision() {
    use super::duodecimal;
    use num::rational::Ratio;
    // 123456 / 1000000 in base 12
    assert_eq!(duodecimal(Ratio::new(296_130, 2_985_984), 5), "12346");
}

#[test]
fn round_zeros() {
    use super::round;
    assert_eq!(round(&[1, 0, 0, 0], true), [1, 0, 0, 1]);
    assert_eq!(round(&[1, 0, 0, 0], false), [1, 0, 0, 0]);
}

#[test]
fn round_elevens() {
    use super::round;
    assert_eq!(round(&[1, 11, 11, 11, 11], true), [2]);
    assert_eq!(round(&[1, 11, 11, 11, 11], false), [1, 11, 11, 11, 11]);
}

#[test]
fn round_to_max() {
    use super::round;
    assert_eq!(round(&[11, 11, 11, 11], true), [12]);
}

#[test]
fn round_at_max() {
    use super::round;
    assert_eq!(round(&[12], true), [12]);
    assert_eq!(round(&[12], false), [12]);
}

#[test]
fn round_over_max() {
    use super::round;
    assert_eq!(round(&[13], true), [12]);
    // assert_eq!(round(&[13], false), [12]); not implemented
}
