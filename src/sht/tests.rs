#[test]
fn sht_value_success() {
    use super::{ChannelRatios, ColourChannel, SecondaryColour, SHT};
    use num::rational::Ratio;
    for (channel_ratios, tint, shade) in [(
        ChannelRatios::OneBrightestChannel {
            primary: ColourChannel::Blue,
            direction_blend: Some((ColourChannel::Red, Ratio::new(1, 2))),
        },
        Ratio::new(1, 2),
        Ratio::new(1, 2),
    )]
    .iter()
    {
        assert_eq!(
            SHT::<u32>::new(*channel_ratios, *tint, *shade),
            Ok(SHT::<u32> {
                channel_ratios: *channel_ratios,
                tint: *tint,
                shade: *shade
            })
        )
    }
}
