//! The RGB color space with 8-bit helpers.
//!
//! Iced's [`Color`] already stores sRGB channels as `f32` in `0.0..=1.0`; this
//! struct exists so all formats share the same conversion API surface
//! (`to_color` / `from_color` / `Display` / `FromStr`) and to provide
//! `const`-friendly 8-bit helpers.

use core::fmt;

use crate::Color;
use crate::const_math::{clamp01, round_for_display};

/// A color in the sRGB color space with an alpha channel.
///
/// All channels are `f32` in `0.0..=1.0`, matching iced's [`Color`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    /// Red component, `0.0..=1.0`.
    pub r: f32,
    /// Green component, `0.0..=1.0`.
    pub g: f32,
    /// Blue component, `0.0..=1.0`.
    pub b: f32,
    /// Alpha (opacity), `0.0..=1.0`.
    pub a: f32,
}

impl Rgb {
    /// Creates a new opaque [`Rgb`] color from normalized channels.
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Sets the alpha (opacity).
    pub const fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    /// Creates an [`Rgb`] color from 8-bit channels (opaque).
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Creates an [`Rgb`] color from 8-bit channels and an 8-bit alpha.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Converts to iced's sRGB [`Color`]. `const`-callable.
    ///
    /// Channels are clamped into `0.0..=1.0`.
    pub const fn to_color(self) -> Color {
        Color::from_rgba(
            clamp01(self.r),
            clamp01(self.g),
            clamp01(self.b),
            clamp01(self.a),
        )
    }

    /// Creates an [`Rgb`] from iced's sRGB [`Color`]. `const`-callable.
    pub const fn from_color(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }

    /// Converts to 8-bit RGBA, rounding and clamping each channel.
    /// `const`-callable.
    pub const fn to_rgba8(self) -> [u8; 4] {
        [
            channel_to_u8(self.r),
            channel_to_u8(self.g),
            channel_to_u8(self.b),
            channel_to_u8(self.a),
        ]
    }
}

const fn channel_to_u8(x: f32) -> u8 {
    let x = if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    };
    (x * 255.0 + 0.5) as u8
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b, _] = self.to_rgba8();
        write!(f, "rgb({r} {g} {b}")?;
        let a = round_for_display(self.a, 100.0);
        if a < 1.0 {
            write!(f, " / {a}")?;
        }
        write!(f, ")")
    }
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        rgb.to_color()
    }
}

impl From<Color> for Rgb {
    fn from(color: Color) -> Self {
        Rgb::from_color(color)
    }
}

#[cfg(test)]
mod tests {
    use std::format;

    use super::*;

    #[test]
    fn from_rgba8_normalizes() {
        let rgb = Rgb::from_rgba8(220, 60, 60, 128);
        assert!((rgb.r - 220.0 / 255.0).abs() < 1e-6);
        assert!((rgb.g - 60.0 / 255.0).abs() < 1e-6);
        assert!((rgb.b - 60.0 / 255.0).abs() < 1e-6);
        assert!((rgb.a - 128.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn rgba8_round_trip() {
        let rgb = Rgb::from_rgba8(255, 128, 0, 7);
        assert_eq!(rgb.to_rgba8(), [255, 128, 0, 7]);
    }

    #[test]
    fn to_rgba8_rounds_and_clamps() {
        assert_eq!(Rgb::new(0.5, 1.0, 0.0).to_rgba8(), [128, 255, 0, 255]);
        assert_eq!(Rgb::new(-1.0, 2.0, 0.999).to_rgba8(), [0, 255, 255, 255]);
    }

    #[test]
    fn color_round_trip() {
        let color = Color::from_rgba8(10, 200, 30, 0.5);
        let rgb = Rgb::from_color(color).to_color();
        assert_eq!(rgb, color);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Rgb::from_rgb8(255, 0, 128)), "rgb(255 0 128)");
        assert_eq!(
            format!("{}", Rgb::from_rgb8(255, 0, 128).with_alpha(0.5)),
            "rgb(255 0 128 / 0.5)"
        );
    }
}
