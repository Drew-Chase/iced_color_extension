//! The OKLAB color space — a perceptually uniform color space by Björn
//! Ottosson.
//!
//! The conversion pipeline is sRGB -> linear -> LMS -> cube root -> OKLAB (and
//! back). The cube root and sRGB gamma are normally non-const operations, so
//! [`crate::const_math`] provides `const fn` replacements, making every
//! function here `const`-callable.
//!
//! Colors outside the sRGB gamut are mapped back by reducing chroma (scaling
//! `a`/`b` toward zero) while preserving lightness, then clamped.

use core::fmt;

use crate::Color;
use crate::const_math::{cbrt, clamp01, linear_to_srgb, round_for_display, srgb_to_linear};

/// How far a linear channel may fall outside `0.0..=1.0` before the color is
/// considered out of the sRGB gamut.
const GAMUT_EPSILON: f32 = 1.0e-4;

/// A color in the OKLAB color space.
///
/// - `l` — perceptual lightness, `0.0` (black) to `1.0` (white)
/// - `a` — green–red opponent axis (negative = green, positive = red)
/// - `b` — blue–yellow opponent axis (negative = blue, positive = yellow)
/// - `alpha` — opacity, `0.0..=1.0`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklab {
    /// Perceptual lightness, `0.0..=1.0`.
    pub l: f32,
    /// Green–red opponent axis.
    pub a: f32,
    /// Blue–yellow opponent axis.
    pub b: f32,
    /// Alpha (opacity), `0.0..=1.0`.
    pub alpha: f32,
}

impl Oklab {
    /// Creates a new opaque [`Oklab`] color.
    pub const fn new(l: f32, a: f32, b: f32) -> Self {
        Self {
            l,
            a,
            b,
            alpha: 1.0,
        }
    }

    /// Sets the alpha (opacity).
    pub const fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// Creates an [`Oklab`] from iced's sRGB [`Color`]. `const`-callable.
    pub const fn from_color(color: Color) -> Self {
        let Color { r, g, b, a: alpha } = color;
        let lr = srgb_to_linear(r);
        let lg = srgb_to_linear(g);
        let lb = srgb_to_linear(b);

        // Linear sRGB -> LMS
        let l_ = 0.412_221_47 * lr + 0.536_332_54 * lg + 0.051_445_993 * lb;
        let m_ = 0.211_903_5 * lr + 0.680_699_5 * lg + 0.107_396_96 * lb;
        let s_ = 0.088_302_46 * lr + 0.281_718_85 * lg + 0.629_978_7 * lb;

        // Nonlinear response (cube root)
        let l_c = cbrt(l_);
        let m_c = cbrt(m_);
        let s_c = cbrt(s_);

        // LMS' -> OKLAB
        Self {
            l: 0.210_454_26 * l_c + 0.793_617_8 * m_c - 0.004_072_047 * s_c,
            a: 1.977_998_5 * l_c - 2.428_592_2 * m_c + 0.450_593_7 * s_c,
            b: 0.025_904_037 * l_c + 0.782_771_77 * m_c - 0.808_675_77 * s_c,
            alpha,
        }
    }

    /// Converts to iced's sRGB [`Color`]. `const`-callable.
    ///
    /// Lightness is clamped to `0.0..=1.0`. If the color falls outside the
    /// sRGB gamut, chroma is reduced (preserving lightness and hue) until it
    /// fits, then channels are clamped.
    pub const fn to_color(self) -> Color {
        let Self { l, a, b, alpha } = self;
        let l = clamp01(l);
        let (r, g, bl) = lab_to_linear_gamut(l, a, b);
        Color::from_rgba(
            linear_to_srgb(clamp01(r)),
            linear_to_srgb(clamp01(g)),
            linear_to_srgb(clamp01(bl)),
            alpha,
        )
    }

    /// Converts to polar OKLCH, returning `(l, chroma, hue_degrees)`.
    ///
    /// This is the same representation iced's `Color::into_oklch` uses. Not
    /// `const` (requires trigonometry).
    pub fn to_oklch(self) -> (f32, f32, f32) {
        let chroma = (self.a * self.a + self.b * self.b).sqrt();
        let mut hue = self.b.atan2(self.a).to_degrees();
        if hue < 0.0 {
            hue += 360.0;
        }
        (self.l, chroma, hue)
    }

    /// Creates an [`Oklab`] from polar OKLCH components
    /// `(l, chroma, hue_degrees)`. Not `const` (requires trigonometry).
    pub fn from_oklch(l: f32, chroma: f32, hue_degrees: f32) -> Self {
        let radians = hue_degrees.to_radians();
        Self::new(l, chroma * radians.cos(), chroma * radians.sin())
    }
}

/// OKLAB -> linear sRGB, without any gamut handling.
const fn lab_to_linear(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    // OKLAB -> LMS'
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

    // Cubing restores the linear response
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // LMS -> linear sRGB
    (
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    )
}

const fn in_gamut(r: f32, g: f32, b: f32) -> bool {
    r >= -GAMUT_EPSILON
        && r <= 1.0 + GAMUT_EPSILON
        && g >= -GAMUT_EPSILON
        && g <= 1.0 + GAMUT_EPSILON
        && b >= -GAMUT_EPSILON
        && b <= 1.0 + GAMUT_EPSILON
}

/// OKLAB -> linear sRGB, reducing chroma when out of the sRGB gamut.
const fn lab_to_linear_gamut(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let (r, g, bl) = lab_to_linear(l, a, b);
    if in_gamut(r, g, bl) {
        return (r, g, bl);
    }
    // Gray (chroma = 0) is always in gamut for `0.0..=1.0` lightness, so a
    // fitting chroma scale is guaranteed to exist. Binary-search the largest.
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    let mut i = 0;
    while i < 16 {
        let mid = (lo + hi) * 0.5;
        let (mr, mg, mb) = lab_to_linear(l, a * mid, b * mid);
        if in_gamut(mr, mg, mb) {
            lo = mid;
        } else {
            hi = mid;
        }
        i += 1;
    }
    lab_to_linear(l, a * lo, b * lo)
}

impl fmt::Display for Oklab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l = round_for_display(self.l, 100_000.0);
        let a = round_for_display(self.a, 100_000.0);
        let b = round_for_display(self.b, 100_000.0);
        let alpha = round_for_display(self.alpha, 100.0);
        write!(f, "oklab({l} {a} {b}")?;
        if alpha < 1.0 {
            write!(f, " / {alpha}")?;
        }
        write!(f, ")")
    }
}

impl From<Oklab> for Color {
    fn from(lab: Oklab) -> Self {
        lab.to_color()
    }
}

impl From<Color> for Oklab {
    fn from(color: Color) -> Self {
        Oklab::from_color(color)
    }
}

#[cfg(test)]
mod tests {
    use std::format;

    use super::*;

    fn close(a: f32, b: f32, what: &str) {
        assert!((a - b).abs() < 1.0e-4, "{what}: {a} vs {b}");
    }

    /// Reference values from Björn Ottosson's OKLAB test data.
    #[test]
    fn reference_values() {
        let cases = [
            (
                Color::from_rgb(1.0, 0.0, 0.0),
                0.627_955,
                0.224_863,
                0.125_846,
            ),
            (
                Color::from_rgb(0.0, 1.0, 0.0),
                0.866_44,
                -0.233_89,
                0.179_50,
            ),
            (
                Color::from_rgb(0.0, 0.0, 1.0),
                0.452_01,
                -0.032_46,
                -0.311_53,
            ),
            (Color::from_rgb(1.0, 1.0, 1.0), 1.0, 0.0, 0.0),
            (Color::from_rgb(0.0, 0.0, 0.0), 0.0, 0.0, 0.0),
        ];
        for (color, l, a, b) in cases {
            let lab = Oklab::from_color(color);
            close(lab.l, l, "l");
            close(lab.a, a, "a");
            close(lab.b, b, "b");
        }
    }

    #[test]
    fn round_trip() {
        let colors = [
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
            Color::from_rgb(1.0, 1.0, 0.0),
            Color::from_rgb(0.0, 1.0, 1.0),
            Color::from_rgb(1.0, 0.0, 1.0),
            Color::from_rgb(0.2, 0.6, 0.9),
            Color::from_rgb(0.1, 0.1, 0.1),
            Color::from_rgb(0.5, 0.5, 0.5),
            Color::from_rgb(1.0, 1.0, 1.0),
            Color::from_rgb(0.0, 0.0, 0.0),
        ];
        for color in colors {
            let back = Oklab::from_color(color).to_color();
            assert!(
                (back.r - color.r).abs() < 2.0e-3
                    && (back.g - color.g).abs() < 2.0e-3
                    && (back.b - color.b).abs() < 2.0e-3,
                "round trip failed for {color:?} -> {back:?}"
            );
            assert!((back.a - color.a).abs() < 1.0e-6);
        }
    }

    #[test]
    fn grays_stay_gray() {
        let mut i = 0;
        while i <= 10 {
            let level = i as f32 / 10.0;
            let lab = Oklab::from_color(Color::from_rgb(level, level, level));
            close(lab.a, 0.0, "a");
            close(lab.b, 0.0, "b");
            i += 1;
        }
    }

    #[test]
    fn out_of_gamut_reduces_chroma_but_keeps_lightness() {
        // A wildly saturated color that cannot exist in sRGB.
        let lab = Oklab::new(0.5, 0.35, 0.2);
        let color = lab.to_color();
        for channel in [color.r, color.g, color.b] {
            assert!(
                (0.0..=1.0).contains(&channel),
                "channel out of range: {channel}"
            );
        }
        let mapped = Oklab::from_color(color);
        close(mapped.l, 0.5, "lightness");
        // Chroma must have been reduced.
        let original_chroma = (lab.a * lab.a + lab.b * lab.b).sqrt();
        let mapped_chroma = (mapped.a * mapped.a + mapped.b * mapped.b).sqrt();
        assert!(mapped_chroma < original_chroma);
    }

    #[test]
    fn lightness_clamped() {
        let color = Oklab::new(2.0, 0.0, 0.0).to_color();
        close(color.r, 1.0, "r");
        close(color.g, 1.0, "g");
        close(color.b, 1.0, "b");

        let color = Oklab::new(-1.0, 0.0, 0.0).to_color();
        close(color.r, 0.0, "r");
        close(color.b, 0.0, "b");
    }

    #[test]
    fn oklch_round_trip() {
        let lab = Oklab::from_color(Color::from_rgb(0.2, 0.6, 0.9));
        let (l, c, h) = lab.to_oklch();
        let back = Oklab::from_oklch(l, c, h);
        close(back.l, lab.l, "l");
        close(back.a, lab.a, "a");
        close(back.b, lab.b, "b");
        assert!((0.0..360.0).contains(&h));
    }

    #[test]
    fn display() {
        let lab = Oklab::from_color(Color::from_rgb(1.0, 0.0, 0.0));
        assert_eq!(format!("{lab}"), "oklab(0.62796 0.22486 0.12585)");
        assert_eq!(
            format!("{}", Oklab::new(0.5, 0.1, -0.1).with_alpha(0.5)),
            "oklab(0.5 0.1 -0.1 / 0.5)"
        );
    }

    #[test]
    fn const_context() {
        const COLOR: Color = Oklab::new(0.5, 0.1, -0.1).to_color();
        assert_eq!(COLOR, Oklab::new(0.5, 0.1, -0.1).to_color());
    }
}
