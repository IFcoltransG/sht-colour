use super::{ChannelRatios, ColourChannel, ParsePropertyError, SecondaryColour, SHT};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take},
    character::complete::digit1,
    combinator::{fail, map, map_res, opt, success, value, verify},
    multi::fold_many1,
    sequence::{pair, tuple},
    Finish, IResult,
};
use num::{rational::Ratio, CheckedAdd, CheckedDiv, CheckedMul, Integer, One, Unsigned, Zero};

pub fn duodecimal_digit(input: &str) -> IResult<&str, &str> {
    // ensure only one digit is taken
    let (input, first) = take(1_u8)(input)?;
    // handle errors
    match alt((tag_no_case("X"), tag_no_case("E"), digit1))(first)? {
        ("", result) => Ok((input, result)),
        _ => fail(input),
    }
}

pub fn number_from_digit<T>(input: &str) -> IResult<&str, T>
where
    u8: Into<T>,
{
    map(
        map_res(duodecimal_digit, |character| match character {
            "E" => Ok(11),
            "X" => Ok(10),
            c => c.parse(),
        }),
        u8::into,
    )(input)
}

pub fn secondary_colour(input: &str) -> IResult<&str, SecondaryColour> {
    alt((
        value(SecondaryColour::Cyan, tag_no_case("c")),
        value(SecondaryColour::Yellow, tag_no_case("y")),
        value(SecondaryColour::Magenta, tag_no_case("m")),
    ))(input)
}

pub fn primary_colour(input: &str) -> IResult<&str, ColourChannel> {
    alt((
        value(ColourChannel::Red, tag_no_case("r")),
        value(ColourChannel::Green, tag_no_case("g")),
        value(ColourChannel::Blue, tag_no_case("b")),
    ))(input)
}

fn try_shift_fraction<T>(base: &T, digit: T, index: u8) -> Option<(u8, Ratio<T>)>
where
    T: Clone + Integer + CheckedMul,
{
    let mut number = Ratio::from_integer(digit);
    let new_index = index.checked_add(1)?;

    for _ in 0..new_index {
        number = number.checked_div(&Ratio::from_integer(base.clone()))?;
    }
    Some((new_index, number))
}

pub fn quantity<T>(input: &str) -> IResult<&str, Ratio<T>>
where
    u8: Into<T>,
    T: CheckedMul + CheckedAdd + Clone + Integer,
{
    let base = 12.into();
    let half_base = || (12 / 2).into(); // for rounding

    // calculate number from digits, and store input precision
    let mut digit_folder = fold_many1(
        number_from_digit,
        || (0_u8, Ratio::from_integer(0.into()), None),
        |(length, number, round_up), digit| {
            try_shift_fraction(&base, digit.clone(), length)
                .and_then(|(length, shifted_digit)| {
                    Some((length, number.checked_add(&shifted_digit)?, None))
                })
                // if unwrapping, it means denominator exceeded maximum size for type
                // so check if we need to round up (unless already calculated)
                .unwrap_or_else(|| (length, number, round_up.or_else(|| Some(digit >= half_base()))))
        },
    );
    let (input, (length, number, round_up)) = digit_folder(input)?;
    match round_up {
        Some(true) => {
            let correction = try_shift_fraction(&base, 1.into(), length - 1).map_or_else(<_>::zero, |(_, n)| n);
            Ok((input, number + correction))
        }
        _ => Ok((input, number)),
    }
}

pub fn direction_blend<T>(input: &str) -> IResult<&str, (ColourChannel, Ratio<T>)>
where
    T: Clone + Integer + CheckedMul + CheckedAdd,
    u8: Into<T>,
{
    let (input, (blend, direction)) = pair(quantity, primary_colour)(input)?;
    Ok((input, (direction, blend)))
}

pub fn channel_ratios<T>(input: &str) -> IResult<&str, ChannelRatios<T>>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned,
    u8: Into<T>,
{
    alt((
        map(
            pair(primary_colour, opt(direction_blend)),
            |(primary, direction_blend)| ChannelRatios::OneBrightestChannel {
                primary,
                direction_blend,
            },
        ),
        map(secondary_colour, |secondary| {
            ChannelRatios::TwoBrightestChannels { secondary }
        }),
    ))(input)
}

type SHTParts<T> = (Option<Ratio<T>>, ChannelRatios<T>, Option<Ratio<T>>);

pub fn sht_data<T>(input: &str) -> IResult<&str, SHTParts<T>>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned,
    u8: Into<T>,
{
    let zero_shade = map(verify(quantity, |v| v.is_zero()), Some);
    let shade = quantity;
    let empty_channel = || success(ChannelRatios::ThreeBrightestChannels);
    let empty_quantity = || success(None);
    let tint = || verify(quantity, |v| !v.is_zero());
    alt((
        // attempt to parse maximally many numeric components
        // separated by colours
        tuple((opt(shade), channel_ratios, opt(tint()))),
        // fall back to parsing one numeric component
        tuple((zero_shade, empty_channel(), empty_quantity())),
        tuple((empty_quantity(), empty_channel(), map(tint(), Some))),
        // special case for duodecimal digit 12
        value(
            (
                None,
                ChannelRatios::ThreeBrightestChannels,
                Some(Ratio::one()),
            ),
            tag_no_case("W"),
        ),
    ))(input)
}

pub fn parse_sht<T>(input: &str) -> Result<SHT<T>, ParsePropertyError>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned,
    u8: Into<T>,
{
    match sht_data(input).finish() {
        Ok(("", (shade, channel_ratios, tint))) => SHT::new(
            channel_ratios,
            shade.unwrap_or_else(<_>::one),
            tint.unwrap_or_else(<_>::zero),
        )
        .map_err(ParsePropertyError::ValueErrors),
        Ok((remaining, _)) => Err(ParsePropertyError::InputRemaining(remaining.to_owned())),
        Err(y) => Err(y.into()),
    }
}
