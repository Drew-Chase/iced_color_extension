#![no_std]
#![warn(missing_docs)]
//! Extended color formats and manipulation for [iced].
//!
//! [iced]'s [`Color`] only ships with hex-string parsing and a handful of
//! runtime helpers. This crate adds the color formats you actually want to
//! work with — HSL, RGB (with 8-bit helpers) and OKLAB — plus perceptual
//! manipulation of lightness, hue, saturation, contrast and alpha. Everything
//! that can be computed at compile time is a `const fn`.
//!
//! # Usage
//!
//! Depend on this crate alongside `iced` (or just `iced_core`) with matching
//! versions. [`iced::Color`](https://docs.rs/iced/latest/iced/struct.Color.html)
//! is a re-export of [`iced_core::Color`], so the blanket implementation
//! applies to both.
//!
//! ```
//! use iced_core::Color;
//! use iced_color_extension::{hsl, rgb, oklab, IcedColorExt};
//!
//! // Compile-time colors — every macro and conversion is a `const fn`:
//! const ACCENT: Color = hsl!(210, 80%, 50%);
//! const DANGER: Color = rgb!(220, 60, 60);
//! const SUCCESS: Color = oklab!(0.72, -0.15, 0.1);
//!
//! // Runtime manipulation through the extension trait:
//! let text = DANGER.lighten(0.15);
//! let text = text.ensure_contrast(ACCENT, 4.5);
//! let hue = text.to_hsl().h;
//! assert!((0.0..360.0).contains(&hue));
//! ```
//!
//! # Compile-time support
//!
//! Stable Rust does not allow `const fn` trait methods, so the trait methods
//! below are runtime-only. For compile-time work, use the const equivalents:
//!
//! - Macros: [`hsl!`], [`rgb!`], [`oklab!`]
//! - Conversion structs: [`Hsl`], [`Rgb`], [`Oklab`] (`new`, `with_alpha`,
//!   `to_color`, `from_color` are all `const fn`)
//! - Free functions: everything in [`ops`]
//!
//! ```
//! use iced_core::Color;
//! use iced_color_extension::{hsl, ops};
//!
//! const MUTED_TEXT: Color = ops::lighten(ops::with_saturation(hsl!(210, 80%, 50%), 0.2), 0.75);
//! ```
//!
//! # Semantics
//!
//! - `lighten`/`darken`/`with_lightness` operate on HSL lightness
//! - `with_luminance`/`adjust_luminance` operate on perceptual OKLAB
//!   lightness, which stays uniform across hues
//! - `relative_luminance`/`contrast_ratio`/`ensure_contrast` follow
//!   [WCAG 2.1](https://www.w3.org/TR/WCAG21/)
//! - `with_contrast` matches CSS `filter: contrast()`
//!
//! OKLAB colors outside the sRGB gamut are mapped back by reducing chroma
//! while preserving lightness and hue.
//!
//! # Parsing and formatting
//!
//! With the default `std` feature, [`Color::parse_css`] parses hex,
//! `rgb()`, `hsl()` and `oklab()` strings (iced only supports hex), and the
//! conversion structs implement CSS-flavored [`Display`] and [`FromStr`].
//!
//! [iced]: https://iced.rs
//! [`Display`]: core::fmt::Display
//! [`FromStr`]: core::str::FromStr

#[cfg(any(feature = "std", test))]
extern crate std;

mod const_math;
mod hsl;
mod macros;
mod oklab;
pub mod ops;
#[cfg(feature = "std")]
pub mod parse;
mod rgb;

pub use crate::hsl::Hsl;
pub use crate::oklab::Oklab;
#[cfg(feature = "std")]
pub use crate::parse::{ParseError, parse_color};
pub use crate::rgb::Rgb;
pub use iced_core::Color;

/// Extensions to iced's [`Color`] for alternative color spaces and perceptual
/// manipulation.
///
/// This trait is implemented for [`Color`] itself; bring it into scope and
/// every method is available on your colors.
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::{hsl, IcedColorExt};
///
/// let accent = hsl!(210.0, 0.8, 0.5);
/// let soft = accent.lighten(0.2).with_alpha(0.5);
/// assert!(soft.to_hsl().l > accent.to_hsl().l);
/// ```
///
/// Note that these methods cannot be `const fn` on stable Rust; see
/// [the crate docs](self#compile-time-support) for the const equivalents.
pub trait IcedColorExt {
    /// Converts to the HSL color space.
    fn to_hsl(self) -> Hsl;
    /// Converts to an [`Rgb`] struct (sRGB with 8-bit helpers).
    fn to_rgb(self) -> Rgb;
    /// Converts to the OKLAB color space.
    fn to_oklab(self) -> Oklab;

    /// Creates a color from HSL components (`h` in degrees, `s`/`l` in
    /// `0.0..=1.0`).
    fn from_hsl(h: f32, s: f32, l: f32) -> Color;
    /// Creates a color from HSL components and an alpha value.
    fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Color;
    /// Creates a color from OKLAB components.
    fn from_oklab(l: f32, a: f32, b: f32) -> Color;
    /// Creates a color from OKLAB components and an alpha value.
    fn from_oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Color;

    /// Sets the hue (in degrees), preserving saturation and lightness.
    fn with_hue(self, hue: f32) -> Color;
    /// Rotates the hue by `delta` degrees (wrapping at 360°).
    fn rotate_hue(self, delta: f32) -> Color;
    /// Sets the saturation, preserving hue and lightness.
    fn with_saturation(self, saturation: f32) -> Color;
    /// Increases saturation by `amount`.
    fn saturate(self, amount: f32) -> Color;
    /// Decreases saturation by `amount`.
    fn desaturate(self, amount: f32) -> Color;
    /// Sets the HSL lightness, preserving hue and saturation.
    fn with_lightness(self, lightness: f32) -> Color;
    /// Increases the HSL lightness by `amount`.
    fn lighten(self, amount: f32) -> Color;
    /// Decreases the HSL lightness by `amount`.
    fn darken(self, amount: f32) -> Color;
    /// Sets the perceptual (OKLAB) lightness, preserving hue and chroma.
    fn with_luminance(self, lightness: f32) -> Color;
    /// Adjusts the perceptual (OKLAB) lightness by `delta`.
    fn adjust_luminance(self, delta: f32) -> Color;
    /// Adjusts contrast like CSS `filter: contrast(f)` (scaling channels
    /// around mid-gray).
    fn with_contrast(self, factor: f32) -> Color;
    /// Sets the alpha (opacity).
    fn with_alpha(self, alpha: f32) -> Color;
    /// Scales the alpha (opacity) by a factor.
    fn scale_alpha(self, factor: f32) -> Color;
    /// Returns the WCAG 2.1 relative luminance (`0.0` — `1.0`).
    fn relative_luminance(self) -> f32;
    /// Returns the WCAG 2.1 contrast ratio against another color (`1.0` —
    /// `21.0`).
    fn contrast_ratio(self, other: Color) -> f32;
    /// Adjusts the perceptual lightness of this color until it reaches
    /// `min_ratio` contrast against `background`, changing it as little as
    /// possible.
    fn ensure_contrast(self, background: Color, min_ratio: f32) -> Color;

    /// Parses a color from hex, `rgb()`, `hsl()`, or `oklab()` notation.
    ///
    /// ```
    /// use iced_core::Color;
    /// use iced_color_extension::IcedColorExt;
    ///
    /// let color = Color::parse_css("hsl(120 50% 50% / 0.5)").unwrap();
    /// assert!((color.g - 0.75).abs() < 1.0e-4);
    /// ```
    #[cfg(feature = "std")]
    fn parse_css(s: &str) -> Result<Color, ParseError>;
}

impl IcedColorExt for Color {
    #[inline]
    fn to_hsl(self) -> Hsl {
        Hsl::from_color(self)
    }

    #[inline]
    fn to_rgb(self) -> Rgb {
        Rgb::from_color(self)
    }

    #[inline]
    fn to_oklab(self) -> Oklab {
        Oklab::from_color(self)
    }

    #[inline]
    fn from_hsl(h: f32, s: f32, l: f32) -> Color {
        Hsl::new(h, s, l).to_color()
    }

    #[inline]
    fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Color {
        Hsl::new(h, s, l).with_alpha(a).to_color()
    }

    #[inline]
    fn from_oklab(l: f32, a: f32, b: f32) -> Color {
        Oklab::new(l, a, b).to_color()
    }

    #[inline]
    fn from_oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Color {
        Oklab::new(l, a, b).with_alpha(alpha).to_color()
    }

    #[inline]
    fn with_hue(self, hue: f32) -> Color {
        ops::with_hue(self, hue)
    }

    #[inline]
    fn rotate_hue(self, delta: f32) -> Color {
        ops::rotate_hue(self, delta)
    }

    #[inline]
    fn with_saturation(self, saturation: f32) -> Color {
        ops::with_saturation(self, saturation)
    }

    #[inline]
    fn saturate(self, amount: f32) -> Color {
        ops::saturate(self, amount)
    }

    #[inline]
    fn desaturate(self, amount: f32) -> Color {
        ops::desaturate(self, amount)
    }

    #[inline]
    fn with_lightness(self, lightness: f32) -> Color {
        ops::with_lightness(self, lightness)
    }

    #[inline]
    fn lighten(self, amount: f32) -> Color {
        ops::lighten(self, amount)
    }

    #[inline]
    fn darken(self, amount: f32) -> Color {
        ops::darken(self, amount)
    }

    #[inline]
    fn with_luminance(self, lightness: f32) -> Color {
        ops::with_luminance(self, lightness)
    }

    #[inline]
    fn adjust_luminance(self, delta: f32) -> Color {
        ops::adjust_luminance(self, delta)
    }

    #[inline]
    fn with_contrast(self, factor: f32) -> Color {
        ops::with_contrast(self, factor)
    }

    #[inline]
    fn with_alpha(self, alpha: f32) -> Color {
        ops::with_alpha(self, alpha)
    }

    #[inline]
    fn scale_alpha(self, factor: f32) -> Color {
        ops::scale_alpha(self, factor)
    }

    #[inline]
    fn relative_luminance(self) -> f32 {
        ops::relative_luminance(self)
    }

    #[inline]
    fn contrast_ratio(self, other: Color) -> f32 {
        ops::contrast_ratio(self, other)
    }

    #[inline]
    fn ensure_contrast(self, background: Color, min_ratio: f32) -> Color {
        ops::ensure_contrast(self, background, min_ratio)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn parse_css(s: &str) -> Result<Color, ParseError> {
        parse::parse_color(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::ops;
    use crate::{Color, Hsl, IcedColorExt, Oklab, Rgb, hsl, oklab, rgb};

    #[test]
    fn macros_work_in_const_contexts() {
        const A: Color = hsl!(210.0, 0.8, 0.5);
        const B: Color = rgb!(220, 60, 60);
        const C: Color = oklab!(0.7, 0.05, -0.02);
        const D: Color = hsl!(210.0, 0.8, 0.5, 0.4);
        assert_eq!(A, hsl!(210.0, 0.8, 0.5));
        assert_eq!(B, Color::from_rgb8(220, 60, 60));
        assert_eq!(C, oklab!(0.7, 0.05, -0.02));
        assert_eq!(D.a, 0.4);
    }

    #[test]
    fn conversions_agree_with_structs() {
        let color = Color::from_rgb8(120, 90, 200);
        assert_eq!(color.to_hsl(), Hsl::from_color(color));
        assert_eq!(color.to_rgb(), Rgb::from_color(color));
        assert_eq!(color.to_oklab(), Oklab::from_color(color));
    }

    #[test]
    fn constructors_agree_with_structs() {
        assert_eq!(
            Color::from_hsl(210.0, 0.8, 0.5),
            Hsl::new(210.0, 0.8, 0.5).to_color()
        );
        assert_eq!(
            Color::from_hsla(210.0, 0.8, 0.5, 0.4),
            Hsl::new(210.0, 0.8, 0.5).with_alpha(0.4).to_color()
        );
        assert_eq!(
            Color::from_oklab(0.7, 0.05, -0.02),
            Oklab::new(0.7, 0.05, -0.02).to_color()
        );
        assert_eq!(
            Color::from_oklaba(0.7, 0.05, -0.02, 0.4),
            Oklab::new(0.7, 0.05, -0.02).with_alpha(0.4).to_color()
        );
    }

    #[test]
    fn trait_methods_agree_with_const_fns() {
        let color = Color::from_rgb8(30, 144, 255);
        assert_eq!(color.lighten(0.1), ops::lighten(color, 0.1));
        assert_eq!(color.rotate_hue(30.0), ops::rotate_hue(color, 30.0));
        assert_eq!(color.with_luminance(0.4), ops::with_luminance(color, 0.4));
        assert_eq!(color.with_contrast(1.5), ops::with_contrast(color, 1.5));
        assert_eq!(
            color.ensure_contrast(Color::WHITE, 4.5),
            ops::ensure_contrast(color, Color::WHITE, 4.5)
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn parse_css_works() {
        let color = Color::parse_css("rgb(30 144 255)").unwrap();
        assert_eq!(color, Color::from_rgb8(30, 144, 255));
        assert!(Color::parse_css("nope").is_err());
    }

    #[test]
    fn full_workflow() {
        let base = hsl!(210.0, 0.8, 0.5);
        let text = base.darken(0.2).ensure_contrast(Color::WHITE, 4.5);
        let hsl = text.to_hsl();
        assert!(hsl.l < 0.5);
        assert!(text.contrast_ratio(Color::WHITE) >= 4.5);
        assert_eq!(text.scale_alpha(0.5).a, text.a * 0.5);
    }
}
