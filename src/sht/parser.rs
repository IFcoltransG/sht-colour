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
use num::{
    pow::Pow, rational::Ratio, CheckedAdd, CheckedDiv, CheckedMul, Integer, One, Unsigned, Zero,
};

pub fn duodecimal_digit(input: &str) -> IResult<&str, &str> {
    // ensure only one digit is taken
    let (input, first) = take(1u8)(input)?;
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
        number = number.checked_div(&Ratio::from_integer(base.clone()))?
    }
    Some((new_index, number))
}

pub fn quantity<T>(input: &str) -> IResult<&str, Ratio<T>>
where
    u8: Into<T>,
    T: CheckedMul + CheckedAdd + Clone + Integer + Pow<T, Output = T>,
{
    let base = 12.into();
    // calculate number from digits, and store input precision
    let mut digit_folder = fold_many1(
        number_from_digit,
        || (0u8, Ratio::from_integer(0.into())),
        |(length, number), digit| {
            try_shift_fraction(&base, digit, length)
                .and_then(|(length, shifted_digit)| {
                    Some((length, number.checked_add(&shifted_digit)?))
                })
                .unwrap_or((length, number))
        },
    );
    let (input, (_, number)) = digit_folder(input)?;
    Ok((input, number))
}

pub fn direction_blend<T>(input: &str) -> IResult<&str, (ColourChannel, Ratio<T>)>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Pow<T, Output = T>,
    u8: Into<T>,
{
    let (input, (blend, direction)) = pair(quantity, primary_colour)(input)?;
    Ok((input, (direction, blend)))
}

pub fn channel_ratios<T>(input: &str) -> IResult<&str, ChannelRatios<T>>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned + Pow<T, Output = T>,
    u8: Into<T>,
{
    alt((
        map(
            pair(|input| primary_colour(input), opt(direction_blend)),
            |(primary, direction_blend)| ChannelRatios::OneBrightestChannel {
                primary,
                direction_blend,
            },
        ),
        map(
            |input| secondary_colour(input),
            |secondary| ChannelRatios::TwoBrightestChannels { secondary },
        ),
    ))(input)
}

pub fn sht_data<T>(
    input: &str,
) -> IResult<&str, (Option<Ratio<T>>, ChannelRatios<T>, Option<Ratio<T>>)>
where
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned + Pow<T, Output = T>,
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
    T: Clone + Integer + CheckedMul + CheckedAdd + Unsigned + Pow<T, Output = T>,
    u8: Into<T>,
{
    match sht_data(input).finish() {
        Ok((_, (shade, channel_ratios, tint))) => SHT::new(
            channel_ratios,
            shade.unwrap_or(<_>::one()),
            tint.unwrap_or(<_>::zero()),
        )
        .map_err(ParsePropertyError::ValueErrors),
        Err(y) => Err(y.into()),
    }
}
