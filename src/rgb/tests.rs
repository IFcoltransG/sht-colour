#[test]
fn parse_success() {
    use super::RGB;
    assert_eq!("#123".parse::<RGB<u8>>(), Ok(RGB::new(1, 2, 3)));
    assert_eq!(
        "#555666777".parse::<RGB<u64>>(),
        Ok(RGB::new(0x555, 0x666, 0x777))
    );
}

#[test]
fn parse_failure() {
    use super::{ParseHexError, RGB};
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
