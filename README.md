# sht-colour
`sht-colour` is for conversions involving SHT colour codes. SHT codes are an intuitive human-readable text format for colours. See <https://omaitzen.com/sht/spec/> for the specification. Supports conversion to and from RGB/hex and parsing from text.

## Example
```rust
use ::sht_colour::{
    rgb::{HexRGB, RGB},
    Ratio, SHT,
};

let red_sht = "r".parse::<SHT<u8>>().unwrap();
let red_hex = "#F00".parse::<HexRGB<u8>>().unwrap();

// `RGB` is the standard struct for RGB values, from the `rgb` crate.
let red_rgb = <RGB<Ratio<u8>>>::new(
    Ratio::from_integer(1),
    Ratio::from_integer(0),
    Ratio::from_integer(0),
);

// Converting between SHT and HexRGB (with a precision of 1 digit).
assert_eq!(red_sht.to_rgb(1), red_hex);
assert_eq!(red_sht, red_hex.to_sht(1));

// Converting between HexRGB and RGB.
assert_eq!(<RGB<Ratio<u8>>>::from(red_hex), red_rgb);
assert_eq!(red_hex, <HexRGB<u8>>::from(red_rgb));
```
