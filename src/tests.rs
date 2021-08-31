//use super::sht;
#[test]
fn rgb_parse_success() {
    use super::rgb::RGB;
    assert_eq!("#123".parse::<RGB<u8>>(), Ok(RGB::new(1, 2, 3)));
    assert_eq!(
        "#555666777".parse::<RGB<u64>>(),
        Ok(RGB::new(0x555, 0x666, 0x777))
    );
}
#[test]
fn rgb_parse_failure() {
    use super::rgb::{ParseHexError, RGB};
    //failure
    assert_eq!("".parse::<RGB<u8>>(), Err(ParseHexError::EmptyCode));
    assert_eq!(
        "111".parse::<RGB<u8>>(),
        Err(ParseHexError::MissingOctothorpe)
    );
    assert_eq!(
        "#11".parse::<RGB<u8>>(),
        Err(ParseHexError::InvalidDigitCount)
    );
    assert_eq!(
        "#G11".parse::<RGB<u8>>(),
        Err(ParseHexError::DigitParseError)
    );
    assert_eq!("#".parse::<RGB<u8>>(), Err(ParseHexError::DigitParseError));
}

#[test]
fn sht_value_failure() {
    use super::sht::{ChannelRatios, ColourChannel, SHTValueError, SecondaryColour, SHT};
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
fn sht_parse_success() {
    use super::sht::{ChannelRatios, ParsePropertyError, SHT};
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
}

#[test]
fn sht_parse_failure() {
    use super::sht::{ParsePropertyError, SHT};
    assert_eq!("".parse::<SHT<u8>>(), Err(ParsePropertyError::EmptyCode));
}
