use super::{ChannelRatios, ColourChannel, SecondaryColour, SHT};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take},
    character::complete::digit1,
    combinator::{map, map_res, opt, value, fail},
    error::{Error, ErrorKind, ParseError},
    multi::fold_many1,
    sequence::pair,
    Err as NomError, IResult,
};
use num::{pow::Pow, rational::Ratio, Integer, Unsigned};

#[derive(PartialEq, Debug)]
pub enum RatioParseError<I> {
    Nom(I, ErrorKind),
    LengthError,
}

impl<I> ParseError<I> for RatioParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<Error<I>> for RatioParseError<I> {
    fn from(err: Error<I>) -> Self {
        match err {
            Error { input, code } => RatioParseError::Nom(input, code),
        }
    }
}

impl<I> From<RatioParseError<I>> for NomError<RatioParseError<I>> {
    fn from(err: RatioParseError<I>) -> Self {
        NomError::Error(err)
    }
}

fn convert_error<T, R, E, A, F>(result: F) -> impl FnOnce(A) -> IResult<T, R, RatioParseError<E>>
where
    F: FnOnce(A) -> IResult<T, R, Error<E>>,
{
    move |input| result(input).map_err(|err| err.map(<_>::into))
}

pub fn duodecimal_digit(input: &str) -> IResult<&str, &str> {
    // ensure only one digit is taken
    let (input, first) = take(1u8)(input)?;
    // handle errors
    match alt((tag("X"), tag("E"), digit1))(first)? {
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

pub fn quantity_with_precision<T>(
    precision: u8,
) -> impl Fn(&str) -> IResult<&str, Ratio<T>, RatioParseError<&str>>
where
    u8: Into<T>,
    T: Clone + Integer + Pow<T, Output = T>,
{
    move |input| {
        let base = 12.into();
        let power = base.clone().pow(precision.into());
        // calculate number from digits, and store input precision
        let digit_folder = fold_many1(
            number_from_digit,
            || (Some(0u8), 0.into()),
            |(count, number), digit| {
                (
                    count.and_then(|t| t.checked_add(1u8)),
                    number * base.clone().into() + digit.into(),
                )
            },
        );
        let (input, (possible_length, number)) = convert_error(digit_folder)(input)?;
        match possible_length {
            Some(u8_length) => {
                let length = u8_length.into();
                let length_power = base.clone().pow(length);
                let truncated =
                    (Ratio::from_integer(number) * Ratio::new(power.clone(), length_power)).trunc();
                let normalised = truncated / power.clone();
                Ok((input, normalised))
            }
            None => Err(NomError::Error(RatioParseError::LengthError)),
        }
    }
}

pub fn direction_blend<T>(
    precision: u8,
) -> impl Fn(&str) -> IResult<&str, (ColourChannel, Ratio<T>), RatioParseError<&str>>
where
    T: Clone + Integer + Pow<T, Output = T>,
    u8: Into<T>,
{
    move |input| {
        let (input, blend) = quantity_with_precision(precision)(input)?;
        let (input, direction) = convert_error(primary_colour)(input)?;
        Ok((input, (direction, blend)))
    }
}

pub fn channel_ratios<T>(
    precision: u8,
) -> impl Fn(&str) -> IResult<&str, ChannelRatios<T>, RatioParseError<&str>>
where
    T: Clone + Integer + Unsigned + Pow<T, Output = T>,
    u8: Into<T>,
{
    move |input| {
        alt((
            map(
                pair(
                    |input| convert_error(primary_colour)(input),
                    opt(direction_blend(precision)),
                ),
                |(primary, direction_blend)| ChannelRatios::OneBrightestChannel {
                    primary,
                    direction_blend,
                },
            ),
            map(
                |input| convert_error(secondary_colour)(input),
                |secondary| ChannelRatios::TwoBrightestChannels { secondary },
            ),
        ))(input)
    }
}

pub fn sht_data<T>(
    input: &str,
) -> IResult<&str, (Option<Ratio<T>>, ChannelRatios<T>, Option<Ratio<T>>)>
where
    T: Clone + Integer + Unsigned + Pow<T, Output = T>,
    u8: Into<T>,
{
    todo!()
}

fn _f() {
    r"
  Each colour component is a single lowercase letter signifying the corresponding colour, one of r, y, g, c, b, or m.
  Each numeric component is a single dozenal digit signifying the corresponding normalized number rounded down to the nearest 12th, with the restriction that numbers never be rounded down to 0. For example, ½ is equal to 6/12 and thus represented as 6. When used this way, a dozenal digit may be referred to as a perdozenage, analogous to a percentage.
  The two extra dozenal digits used to represent 10/12 and 11/12 are the capital letters X and E.
  An additional capital letter W is used to represent 1 (i.e. 12/12), in place of 10. This is only necessary for the component <tint>, and only if it appears by itself. The effect of this is that W just means pure white.
  If additional precision is useful beyond an integer perdozenage, an SHT code may use a double-digit pergrossage in place of it. Further precision can be introduced analogously with a triple-digit perzagierage, and so on.";
}
