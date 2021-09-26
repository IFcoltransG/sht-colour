#[test]
fn parse_success() {
    use super::RGB;
    use num::rational::Ratio;
    assert_eq!(
        "#123".parse::<RGB<u8>>(),
        Ok(RGB::new(
            Ratio::new(1, 15),
            Ratio::new(2, 15),
            Ratio::new(3, 15)
        ))
    );
    assert_eq!(
        "#555666777".parse::<RGB<u64>>(),
        Ok(RGB::new(
            Ratio::new(0x555, 0xFFF),
            Ratio::new(0x666, 0xFFF),
            Ratio::new(0x777, 0xFFF),
        ))
    );
}

#[test]
fn parse_failure() {
    use super::{ParseHexError, RGB};
    // failure
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
fn display() {
    use super::RGB;
    assert_eq!(
        &format!("{:4}", "#000".parse::<RGB<u32>>().unwrap()),
        "#000000000000"
    );
    assert_eq!(
        &format!("{}", "#ABC".parse::<RGB<u16>>().unwrap()),
        "#AABBCC"
    );
    assert_eq!(
        &format!("{:1}", "#AABBCC".parse::<RGB<u32>>().unwrap()),
        "#ABC"
    );
    assert_eq!(
        &format!("{:4}", "#123456".parse::<RGB<u32>>().unwrap()),
        "#121234345656"
    );
}
