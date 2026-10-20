//! Runtime parsing of CSS-style color strings (requires the `std` feature).
//!
//! Supported notations:
//! - Hex: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` (case-insensitive)
//! - `rgb()` / `rgba()`: legacy comma syntax (`rgb(255, 0, 128)`) and modern
//!   space syntax (`rgb(255 0 128 / 50%)`), channels as `0..255` or percentages
//! - `hsl()` / `hsla()`: same syntax; hue accepts an optional `deg` suffix,
//!   saturation/lightness as percentages or `0.0..=1.0`
//! - `oklab()`: `oklab(L a b / alpha)`; `L` as `0.0..1.0` or a percentage,
//!   `a`/`b` as numbers (or percentages, where `100%` = `0.4`)
//!
//! Components are clamped to their valid ranges, mirroring CSS behavior.

use std::str::FromStr;
use std::vec::Vec;

use crate::const_math::clamp01;
use crate::{Color, Hsl, Oklab, Rgb};

/// An error returned when parsing a color string fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The string is not a recognized color notation.
    UnknownFormat,
    /// A component of the color is missing or malformed.
    InvalidNumber,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFormat => write!(f, "unrecognized color format"),
            Self::InvalidNumber => write!(f, "missing or malformed color component"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a [`Color`] from hex, `rgb()`/`rgba()`, `hsl()`/`hsla()`, or
/// `oklab()` notation.
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::parse_color;
///
/// let red = parse_color("hsl(0 100% 50%)").unwrap();
/// assert_eq!(red, Color::from_rgb(1.0, 0.0, 0.0));
/// ```
pub fn parse_color(input: &str) -> Result<Color, ParseError> {
    let lowered = input.trim().to_ascii_lowercase();
    if let Some(hex) = lowered.strip_prefix('#') {
        return parse_hex(hex);
    }
    let (name, inner) = split_function(&lowered)?;
    match name {
        "rgb" | "rgba" => Ok(parse_rgb(inner)?.to_color()),
        "hsl" | "hsla" => Ok(parse_hsl(inner)?.to_color()),
        "oklab" => Ok(parse_oklab(inner)?.to_color()),
        _ => Err(ParseError::UnknownFormat),
    }
}

fn split_function(s: &str) -> Result<(&str, &str), ParseError> {
    let open = s.find('(').ok_or(ParseError::UnknownFormat)?;
    let close = s.rfind(')').ok_or(ParseError::UnknownFormat)?;
    if open == 0 || close < open || !s[close + 1..].trim().is_empty() {
        return Err(ParseError::UnknownFormat);
    }
    Ok((s[..open].trim(), &s[open + 1..close]))
}

/// Splits the inside of a color function into main tokens and an optional
/// modern-syntax alpha token (after `/`).
fn split_tokens(inner: &str) -> Result<(Vec<&str>, Option<&str>), ParseError> {
    if let Some((main, alpha)) = inner.split_once('/') {
        if main.contains(',') {
            // Mixing legacy commas with modern `/ alpha` is not a valid format.
            return Err(ParseError::UnknownFormat);
        }
        Ok((token_list(main)?, Some(alpha)))
    } else {
        Ok((token_list(inner)?, None))
    }
}

fn token_list(inner: &str) -> Result<Vec<&str>, ParseError> {
    Ok(if inner.contains(',') {
        inner.split(',').collect()
    } else {
        inner.split_whitespace().collect()
    })
}

/// Parses a number, returning its value and whether it was a percentage.
fn number(token: &str) -> Result<(f32, bool), ParseError> {
    let t = token.trim();
    if let Some(pct) = t.strip_suffix('%') {
        let v: f32 = pct.trim().parse().map_err(|_| ParseError::InvalidNumber)?;
        Ok((v, true))
    } else {
        let v: f32 = t.parse().map_err(|_| ParseError::InvalidNumber)?;
        Ok((v, false))
    }
}

/// Parses a hue in degrees, allowing an optional `deg` suffix.
fn hue(token: &str) -> Result<f32, ParseError> {
    let t = token.trim();
    let t = t.strip_suffix("deg").unwrap_or(t);
    let (v, is_pct) = number(t)?;
    if is_pct {
        return Err(ParseError::InvalidNumber);
    }
    Ok(v)
}

/// Parses an alpha value (`0.0..1.0` or a percentage).
fn alpha_value(token: &str) -> Result<f32, ParseError> {
    let (v, pct) = number(token)?;
    Ok(clamp01(if pct { v / 100.0 } else { v }))
}

/// Resolves alpha for legacy 4-component syntax (`rgba(r, g, b, a)`) as well
/// as modern `/ alpha` and 3-component opaque forms.
fn trailing_alpha(tokens: &mut Vec<&str>, explicit: Option<&str>) -> Result<f32, ParseError> {
    if let Some(t) = explicit {
        return alpha_value(t);
    }
    match tokens.len() {
        3 => Ok(1.0),
        4 => {
            let t = tokens[3];
            tokens.truncate(3);
            alpha_value(t)
        }
        _ => Err(ParseError::InvalidNumber),
    }
}

fn parse_rgb(inner: &str) -> Result<Rgb, ParseError> {
    let (mut tokens, alpha) = split_tokens(inner)?;
    let a = trailing_alpha(&mut tokens, alpha)?;
    if tokens.len() != 3 {
        return Err(ParseError::InvalidNumber);
    }
    let channel = |t: &str| -> Result<f32, ParseError> {
        let (v, pct) = number(t)?;
        Ok(clamp01(if pct { v / 100.0 } else { v / 255.0 }))
    };
    Ok(Rgb::new(
        channel(tokens[0])?,
        channel(tokens[1])?,
        channel(tokens[2])?,
    )
    .with_alpha(a))
}

fn parse_hsl(inner: &str) -> Result<Hsl, ParseError> {
    let (mut tokens, alpha) = split_tokens(inner)?;
    let a = trailing_alpha(&mut tokens, alpha)?;
    if tokens.len() != 3 {
        return Err(ParseError::InvalidNumber);
    }
    let h = hue(tokens[0])?;
    let (sv, sp) = number(tokens[1])?;
    let (lv, lp) = number(tokens[2])?;
    let s = clamp01(if sp { sv / 100.0 } else { sv });
    let l = clamp01(if lp { lv / 100.0 } else { lv });
    Ok(Hsl::new(h, s, l).with_alpha(a))
}

fn parse_oklab(inner: &str) -> Result<Oklab, ParseError> {
    let (mut tokens, alpha) = split_tokens(inner)?;
    let a = trailing_alpha(&mut tokens, alpha)?;
    if tokens.len() != 3 {
        return Err(ParseError::InvalidNumber);
    }
    let (lv, lp) = number(tokens[0])?;
    let l = clamp01(if lp { lv / 100.0 } else { lv });
    // In CSS Color 4, 100% of a/b corresponds to ±0.4.
    let (av, ap) = number(tokens[1])?;
    let oa = if ap { av / 100.0 * 0.4 } else { av };
    let (bv, bp) = number(tokens[2])?;
    let ob = if bp { bv / 100.0 * 0.4 } else { bv };
    Ok(Oklab::new(l, oa, ob).with_alpha(a))
}

fn parse_hex(hex: &str) -> Result<Color, ParseError> {
    if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseError::InvalidNumber);
    }
    let v = u32::from_str_radix(hex, 16).map_err(|_| ParseError::InvalidNumber)?;
    let channel8 = |shift: u32| ((v >> shift) & 0xFF) as u8;
    let nibble = |shift: u32| (((v >> shift) & 0xF) as u8).wrapping_mul(17);
    match hex.len() {
        3 => Ok(Color::from_rgb8(nibble(8), nibble(4), nibble(0))),
        4 => Ok(Color::from_rgba8(
            nibble(12),
            nibble(8),
            nibble(4),
            f32::from((v & 0xF) as u8) * 17.0 / 255.0,
        )),
        6 => Ok(Color::from_rgb8(channel8(16), channel8(8), channel8(0))),
        8 => Ok(Color::from_rgba8(
            channel8(24),
            channel8(16),
            channel8(8),
            f32::from((v & 0xFF) as u8) / 255.0,
        )),
        _ => Err(ParseError::UnknownFormat),
    }
}

pub(crate) fn parse_hsl_string(s: &str) -> Result<Hsl, ParseError> {
    let lowered = s.trim().to_ascii_lowercase();
    let (name, inner) = split_function(&lowered)?;
    match name {
        "hsl" | "hsla" => parse_hsl(inner),
        _ => Err(ParseError::UnknownFormat),
    }
}

pub(crate) fn parse_rgb_string(s: &str) -> Result<Rgb, ParseError> {
    let lowered = s.trim().to_ascii_lowercase();
    let (name, inner) = split_function(&lowered)?;
    match name {
        "rgb" | "rgba" => parse_rgb(inner),
        _ => Err(ParseError::UnknownFormat),
    }
}

pub(crate) fn parse_oklab_string(s: &str) -> Result<Oklab, ParseError> {
    let lowered = s.trim().to_ascii_lowercase();
    let (name, inner) = split_function(&lowered)?;
    match name {
        "oklab" => parse_oklab(inner),
        _ => Err(ParseError::UnknownFormat),
    }
}

impl FromStr for Hsl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_hsl_string(s)
    }
}

impl FromStr for Rgb {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_rgb_string(s)
    }
}

impl FromStr for Oklab {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_oklab_string(s)
    }
}

#[cfg(test)]
mod tests {
    use std::format;

    use super::*;

    fn close(a: f32, b: f32, what: &str) {
        assert!((a - b).abs() < 1.0e-4, "{what}: {a} vs {b}");
    }

    #[test]
    fn hex_forms() {
        let red = parse_color("#FF0000").unwrap();
        close(red.r, 1.0, "r");
        close(red.g, 0.0, "g");
        close(red.b, 0.0, "b");
        assert_eq!(red.a, 1.0);

        let red = parse_color("#f00").unwrap();
        close(red.r, 1.0, "r");
        close(red.b, 0.0, "b");

        let half_red = parse_color("#f000").unwrap();
        close(half_red.a, 0.0, "a");

        let dim = parse_color("#8040c080").unwrap();
        close(dim.r, 128.0 / 255.0, "r");
        close(dim.a, 128.0 / 255.0, "a");
    }

    #[test]
    fn rgb_forms() {
        let color = parse_color("rgb(255, 0, 128)").unwrap();
        close(color.r, 1.0, "r");
        close(color.g, 0.0, "g");
        close(color.b, 128.0 / 255.0, "b");
        assert_eq!(color.a, 1.0);

        let color = parse_color("rgb(100% 0% 50%)").unwrap();
        close(color.r, 1.0, "r");
        close(color.b, 0.5, "b");

        let color = parse_color("rgba(255, 0, 0, 0.5)").unwrap();
        close(color.a, 0.5, "a");

        let color = parse_color("rgb(255 0 0 / 50%)").unwrap();
        close(color.a, 0.5, "a");

        let color = parse_color("RGB ( 255 , 0 , 0 )").unwrap();
        close(color.r, 1.0, "r");
    }

    #[test]
    fn hsl_forms() {
        let color = parse_color("hsl(120, 50%, 50%)").unwrap();
        close(color.r, 0.25, "r");
        close(color.g, 0.75, "g");
        close(color.b, 0.25, "b");

        let color = parse_color("hsl(120deg 50% 50%)").unwrap();
        close(color.g, 0.75, "g");

        let color = parse_color("hsla(120, 50%, 50%, 0.25)").unwrap();
        close(color.a, 0.25, "a");

        // Non-percentage saturation/lightness are accepted as 0..1.
        let color = parse_color("hsl(120 0.5 0.5)").unwrap();
        close(color.g, 0.75, "g");
    }

    #[test]
    fn oklab_forms() {
        let color = parse_color("oklab(0.5 0.1 -0.1)").unwrap();
        close(color.a, 1.0, "alpha");

        let color = parse_color("oklab(50% 0.1 -0.1 / 0.5)").unwrap();
        close(color.a, 0.5, "alpha");

        let lab = "oklab(50% 25% -25%)".parse::<Oklab>().unwrap();
        close(lab.l, 0.5, "l");
        close(lab.a, 0.1, "a");
        close(lab.b, -0.1, "b");
    }

    #[test]
    fn display_round_trip() {
        let hsl: Hsl = "hsl(210 80% 50%)".parse().unwrap();
        assert_eq!(format!("{hsl}"), "hsl(210 80% 50%)");

        let rgb: Rgb = "rgb(255 0 128)".parse().unwrap();
        assert_eq!(format!("{rgb}"), "rgb(255 0 128)");

        let lab: Oklab = "oklab(0.62796 0.22486 0.12585)".parse().unwrap();
        assert_eq!(format!("{lab}"), "oklab(0.62796 0.22486 0.12585)");
    }

    #[test]
    fn components_are_clamped() {
        let color = parse_color("rgb(300 -20 128 / 2)").unwrap();
        close(color.r, 1.0, "r");
        close(color.g, 0.0, "g");
        close(color.a, 1.0, "a");
    }

    #[test]
    fn errors() {
        for input in [
            "",
            "#",
            "#GGG",
            "#12345",
            "rgb(",
            "rgb(1, 2)",
            "rgb(1, 2, 3, 4, 5)",
            "rgb(a, b, c)",
            "rgb(1 2 3, 0.5)",
            "foo(1, 2, 3)",
            "hsl(50%, 50%, 50%)",
            "oklab()",
            "rgb(1, 2, 3) extra",
        ] {
            assert!(parse_color(input).is_err(), "expected error for {input:?}");
        }
    }

    #[test]
    fn parse_color_matches_parse_functions() {
        let color = parse_color("hsl(210 80% 50%)").unwrap();
        let hsl: Hsl = "hsl(210 80% 50%)".parse().unwrap();
        assert_eq!(color, hsl.to_color());
    }
}
