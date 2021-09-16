#[test]
fn rgb_to_sht() {
    use super::{rgb::RGB, rgb_to_sht, sht::SHT};
    for (input, output) in [
        ("#ff0000", "r"),
        ("#aa0000", "8r"),
        ("#ff4040", "r3"),
        ("#c04040", "8r3"), // this case mismatches website
        ("#ff8000", "r6g"),
        ("#aa5500", "8r6g"),
        ("#c08040", "8r6g3"),
        ("#c0c040", "8y3"),
        ("#808080", "6"),
        ("#000000", "0"),
        ("#ffffff", "W"),
    ]
    .iter()
    {
        assert_eq!(
            rgb_to_sht(input.parse::<RGB<u32>>().unwrap(), 2),
            output.parse::<SHT<u32>>().unwrap()
        );
    }
}

#[test]
fn sht_to_rgb() {
    use super::{rgb::RGB, sht::SHT, sht_to_rgb};
    for (input, output) in [
        ("r", "#ff0000"),
        ("8r", "#aa0000"),
        ("r3", "#ff4040"),
        ("8r3", "#c04040"), // this case mismatches website
        ("r6g", "#ff8000"),
        ("8r6g", "#aa5500"),
        ("8r6g3", "#c08040"),
        ("8y3", "#c0c040"),
        ("6", "#808080"),
        ("0", "#000000"),
        ("W", "#ffffff"),
    ]
    .iter()
    {
        assert_eq!(
            sht_to_rgb(input.parse::<SHT<u32>>().unwrap(), 2),
            output.parse::<RGB<u32>>().unwrap()
        )
    }
}

#[test]
fn rounding() {
    use super::round_denominator;
    use num::rational::Ratio;
    assert_eq!(
        round_denominator::<u8>(Ratio::new(2, 3), 2, 2),
        Ratio::new(3, 4)
    );
    assert_eq!(
        round_denominator::<u8>(Ratio::new(1, 100), 2, 1),
        Ratio::new(0, 1)
    );
    assert_eq!(
        round_denominator::<u8>(Ratio::new(22, 100), 3, 2),
        Ratio::new(2, 9)
    );
    assert_eq!(
        round_denominator::<u8>(Ratio::new(49, 100), 100, 0),
        Ratio::new(0, 1)
    );
    assert_eq!(
        round_denominator::<u8>(Ratio::new(50, 100), 100, 0),
        Ratio::new(1, 1)
    );
    assert_eq!(
        round_denominator::<u32>(Ratio::new(0, 100), 100, 2),
        Ratio::new(0, 1)
    );
    assert_eq!(
        round_denominator::<u32>(Ratio::new(100, 100), 100, 2),
        Ratio::new(1, 1)
    );
}
