//! The HSL color space (hue, saturation, lightness).
//!
//! Conversions between HSL and sRGB are pure arithmetic, so every function in
//! this module is a `const fn` and can be used to build compile-time colors.

use core::fmt;

use crate::Color;
use crate::const_math::{abs, clamp01, normalize_hue, round_for_display};

/// A color in the HSL color space.
///
/// - `h` — hue in degrees; values outside `0.0..360.0` are wrapped
/// - `s` — saturation, `0.0..=1.0`
/// - `l` — lightness, `0.0..=1.0`
/// - `a` — alpha (opacity), `0.0..=1.0`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// Hue in degrees, wrapped into `0.0..360.0`.
    pub h: f32,
    /// Saturation, `0.0..=1.0`.
    pub s: f32,
    /// Lightness, `0.0..=1.0`.
    pub l: f32,
    /// Alpha (opacity), `0.0..=1.0`.
    pub a: f32,
}

impl Hsl {
    /// Creates a new opaque [`Hsl`] color.
    pub const fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l, a: 1.0 }
    }

    /// Sets the alpha (opacity).
    pub const fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    /// Creates an [`Hsl`] from iced's sRGB [`Color`]. `const`-callable.
    ///
    /// Achromatic colors are given a hue of `0.0`. Inputs outside the sRGB
    /// gamut produce undefined saturation/lightness values.
    pub const fn from_color(color: Color) -> Self {
        let Color { r, g, b, a } = color;
        let max = if r > g {
            if r > b { r } else { b }
        } else if g > b {
            g
        } else {
            b
        };
        let min = if r < g {
            if r < b { r } else { b }
        } else if g < b {
            g
        } else {
            b
        };
        let l = (max + min) * 0.5;
        let d = max - min;
        if d <= 0.0 {
            return Self {
                h: 0.0,
                s: 0.0,
                l,
                a,
            };
        }
        let s = d / (1.0 - abs(2.0 * l - 1.0));
        let h = if max == r {
            60.0 * ((g - b) / d)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        Self {
            h: normalize_hue(h),
            s,
            l,
            a,
        }
    }

    /// Converts to iced's sRGB [`Color`]. `const`-callable.
    pub const fn to_color(self) -> Color {
        let Self { h, s, l, a } = self;
        let s = clamp01(s);
        let l = clamp01(l);
        let h = normalize_hue(h);
        let c = (1.0 - abs(2.0 * l - 1.0)) * s;
        let hp = h / 60.0;
        let sextant = hp as i32;
        // hp mod 2, computed without `f32::rem` (which is not const-stable).
        let m2 = hp - 2.0 * (sextant / 2) as f32;
        let x = c * (1.0 - abs(m2 - 1.0));
        let (r1, g1, b1) = if sextant <= 0 {
            (c, x, 0.0)
        } else if sextant == 1 {
            (x, c, 0.0)
        } else if sextant == 2 {
            (0.0, c, x)
        } else if sextant == 3 {
            (0.0, x, c)
        } else if sextant == 4 {
            (x, 0.0, c)
        } else {
            // sextant 5 (or an edge-case 6 from rounding): red again
            (c, 0.0, x)
        };
        let m = l - c * 0.5;
        Color::from_rgba(r1 + m, g1 + m, b1 + m, clamp01(a))
    }
}

impl fmt::Display for Hsl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = round_for_display(self.h, 100.0);
        let s = round_for_display(self.s * 100.0, 100.0);
        let l = round_for_display(self.l * 100.0, 100.0);
        let a = round_for_display(self.a, 100.0);
        write!(f, "hsl({h} {s}% {l}%")?;
        if a < 1.0 {
            write!(f, " / {a}")?;
        }
        write!(f, ")")
    }
}

impl From<Hsl> for Color {
    fn from(hsl: Hsl) -> Self {
        hsl.to_color()
    }
}

impl From<Color> for Hsl {
    fn from(color: Color) -> Self {
        Hsl::from_color(color)
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
    fn known_values() {
        let cases = [
            (Color::from_rgb(1.0, 0.0, 0.0), 0.0, 1.0, 0.5),
            (Color::from_rgb(0.0, 1.0, 0.0), 120.0, 1.0, 0.5),
            (Color::from_rgb(0.0, 0.0, 1.0), 240.0, 1.0, 0.5),
            (Color::from_rgb(0.0, 1.0, 1.0), 180.0, 1.0, 0.5),
            (Color::from_rgb(1.0, 0.0, 1.0), 300.0, 1.0, 0.5),
            (Color::from_rgb(1.0, 1.0, 0.0), 60.0, 1.0, 0.5),
            (Color::from_rgb(0.2, 0.6, 1.0), 210.0, 1.0, 0.6),
        ];
        for (color, h, s, l) in cases {
            let hsl = Hsl::from_color(color);
            close(hsl.h, h, "h");
            close(hsl.s, s, "s");
            close(hsl.l, l, "l");
        }
    }

    #[test]
    fn achromatic_has_zero_saturation() {
        for level in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let hsl = Hsl::from_color(Color::from_rgb(level, level, level));
            close(hsl.s, 0.0, "s");
            close(hsl.l, level, "l");
        }
    }

    #[test]
    fn round_trip() {
        let colors = [
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
            Color::from_rgb(0.2, 0.6, 0.9),
            Color::from_rgb(0.9, 0.3, 0.5),
            Color::from_rgb(0.0, 0.0, 0.0),
            Color::from_rgb(1.0, 1.0, 1.0),
            Color::from_rgb(0.5, 0.5, 0.5),
        ];
        for color in colors {
            let back = Hsl::from_color(color).to_color();
            close(back.r, color.r, "r");
            close(back.g, color.g, "g");
            close(back.b, color.b, "b");
            assert!((back.a - color.a).abs() < 1.0e-6);
        }
    }

    #[test]
    fn hue_wraps() {
        let blue = Hsl::new(270.0, 1.0, 0.5).to_color();
        assert_eq!(Hsl::new(-90.0, 1.0, 0.5).to_color(), blue);
        assert_eq!(Hsl::new(630.0, 1.0, 0.5).to_color(), blue);
        assert_eq!(
            Hsl::new(360.0, 1.0, 0.5).to_color(),
            Hsl::new(0.0, 1.0, 0.5).to_color()
        );
    }

    #[test]
    fn known_to_color_values() {
        let color = Hsl::new(120.0, 0.5, 0.5).to_color();
        close(color.r, 0.25, "r");
        close(color.g, 0.75, "g");
        close(color.b, 0.25, "b");

        let color = Hsl::new(210.0, 0.8, 0.5).to_color();
        close(color.r, 0.1, "r");
        close(color.g, 0.5, "g");
        close(color.b, 0.9, "b");
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Hsl::new(210.0, 0.8, 0.5)), "hsl(210 80% 50%)");
        assert_eq!(
            format!("{}", Hsl::new(210.0, 0.8, 0.5).with_alpha(0.5)),
            "hsl(210 80% 50% / 0.5)"
        );
    }

    #[test]
    fn const_context() {
        const COLOR: Color = Hsl::new(120.0, 0.5, 0.5).to_color();
        assert_eq!(COLOR, Hsl::new(120.0, 0.5, 0.5).to_color());
    }
}
