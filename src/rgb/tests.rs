#[test]
fn parse_success() {
    use super::HexRGB;
    use ::num::rational::Ratio;
    assert_eq!(
        "#123".parse::<HexRGB<u8>>(),
        Ok(HexRGB::new(
            Ratio::new(1, 15),
            Ratio::new(2, 15),
            Ratio::new(3, 15)
        ))
    );
    assert_eq!(
        "#555666777".parse::<HexRGB<u64>>(),
        Ok(HexRGB::new(
            Ratio::new(0x555, 0xFFF),
            Ratio::new(0x666, 0xFFF),
            Ratio::new(0x777, 0xFFF),
        ))
    );
}

#[test]
fn parse_failure() {
    use super::{ParseHexError, HexRGB};
    // failure
    assert_eq!("".parse::<HexRGB<u8>>(), Err(ParseHexError::EmptyCode));
    assert_eq!(
        "111".parse::<HexRGB<u8>>(),
        Err(ParseHexError::MissingOctothorpe)
    );
    assert_eq!(
        "#11".parse::<HexRGB<u8>>(),
        Err(ParseHexError::InvalidDigitCount)
    );
    assert_eq!(
        "#G11".parse::<HexRGB<u8>>(),
        Err(ParseHexError::DigitParseError)
    );
    assert_eq!("#".parse::<HexRGB<u8>>(), Err(ParseHexError::DigitParseError));
}

#[test]
fn display() {
    use super::HexRGB;
    assert_eq!(
        &format!("{:4}", "#000".parse::<HexRGB<u32>>().unwrap()),
        "#000000000000"
    );
    assert_eq!(
        &format!("{}", "#ABC".parse::<HexRGB<u16>>().unwrap()),
        "#AABBCC"
    );
    assert_eq!(
        &format!("{:1}", "#AABBCC".parse::<HexRGB<u32>>().unwrap()),
        "#ABC"
    );
    assert_eq!(
        &format!("{:4}", "#123456".parse::<HexRGB<u32>>().unwrap()),
        "#121234345656"
    );
}
