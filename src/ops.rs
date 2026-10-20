//! `const fn` color manipulation operations.
//!
//! Every function here is a `const fn`, so it can be used to derive colors at
//! compile time. The [`IcedColorExt`](crate::IcedColorExt) trait mirrors these
//! as ergonomic methods (trait methods cannot be `const fn` on stable Rust).
//!
//! ```
//! use iced_core::Color;
//! use iced_color_extension::ops;
//!
//! const MUTED_TEXT: Color = ops::lighten(ops::with_saturation(Color::BLACK, 0.0), 0.65);
//! ```

use crate::Color;
use crate::const_math::{clamp01, normalize_hue, srgb_to_linear};
use crate::hsl::Hsl;
use crate::oklab::Oklab;

/// Sets the alpha (opacity) of a color.
#[inline]
pub const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, alpha)
}

/// Scales the alpha (opacity) of a color by a factor.
#[inline]
pub const fn scale_alpha(color: Color, factor: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, color.a * factor)
}

/// Sets the hue (in degrees) of a color, preserving saturation and lightness.
///
/// Achromatic colors have no meaningful hue; rotating them is a no-op.
#[inline]
pub const fn with_hue(color: Color, hue: f32) -> Color {
    let mut hsl = Hsl::from_color(color);
    hsl.h = hue;
    hsl.to_color()
}

/// Rotates the hue of a color by `delta` degrees (wrapping at 360°).
///
/// Achromatic colors have no meaningful hue; rotating them is a no-op.
#[inline]
pub const fn rotate_hue(color: Color, delta: f32) -> Color {
    let mut hsl = Hsl::from_color(color);
    hsl.h = normalize_hue(hsl.h + delta);
    hsl.to_color()
}

/// Sets the saturation of a color, preserving hue and lightness.
#[inline]
pub const fn with_saturation(color: Color, saturation: f32) -> Color {
    let mut hsl = Hsl::from_color(color);
    hsl.s = clamp01(saturation);
    hsl.to_color()
}

/// Increases saturation by `amount` (clamped to `1.0`).
///
/// Fully desaturated colors have a hue of 0°, so saturating them tints them red.
#[inline]
pub const fn saturate(color: Color, amount: f32) -> Color {
    with_saturation(color, Hsl::from_color(color).s + amount)
}

/// Decreases saturation by `amount` (clamped to `0.0`).
#[inline]
pub const fn desaturate(color: Color, amount: f32) -> Color {
    with_saturation(color, Hsl::from_color(color).s - amount)
}

/// Sets the HSL lightness of a color, preserving hue and saturation.
#[inline]
pub const fn with_lightness(color: Color, lightness: f32) -> Color {
    let mut hsl = Hsl::from_color(color);
    hsl.l = clamp01(lightness);
    hsl.to_color()
}

/// Lightens a color by increasing its HSL lightness by `amount`.
#[inline]
pub const fn lighten(color: Color, amount: f32) -> Color {
    with_lightness(color, Hsl::from_color(color).l + amount)
}

/// Darkens a color by decreasing its HSL lightness by `amount`.
#[inline]
pub const fn darken(color: Color, amount: f32) -> Color {
    with_lightness(color, Hsl::from_color(color).l - amount)
}

/// Sets the perceptual lightness (OKLAB `l`) of a color.
///
/// Unlike HSL lightness, OKLAB lightness is perceptually uniform: `0.5` is
/// mid-gray regardless of hue. Colors outside the sRGB gamut are mapped back
/// by reducing chroma.
#[inline]
pub const fn with_luminance(color: Color, lightness: f32) -> Color {
    let mut lab = Oklab::from_color(color);
    lab.l = clamp01(lightness);
    lab.to_color()
}

/// Adjusts the perceptual lightness (OKLAB `l`) of a color by `delta`.
#[inline]
pub const fn adjust_luminance(color: Color, delta: f32) -> Color {
    with_luminance(color, Oklab::from_color(color).l + delta)
}

/// Adjusts contrast like CSS `filter: contrast(f)`: channels are scaled
/// around mid-gray as `(channel - 0.5) * f + 0.5` and clamped.
///
/// `1.0` leaves the color unchanged, `0.0` produces mid-gray, values above
/// `1.0` increase contrast, values below `1.0` decrease it.
#[inline]
pub const fn with_contrast(color: Color, factor: f32) -> Color {
    Color::from_rgba(
        clamp01((color.r - 0.5) * factor + 0.5),
        clamp01((color.g - 0.5) * factor + 0.5),
        clamp01((color.b - 0.5) * factor + 0.5),
        color.a,
    )
}

/// Returns the WCAG 2.1 relative luminance of a color (`0.0` — `1.0`).
#[inline]
pub const fn relative_luminance(color: Color) -> f32 {
    let r = srgb_to_linear(color.r);
    let g = srgb_to_linear(color.g);
    let b = srgb_to_linear(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Returns the WCAG 2.1 contrast ratio between two colors (`1.0` — `21.0`).
#[inline]
pub const fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Adjusts the perceptual lightness of `fg` until it reaches `min_ratio`
/// contrast against `bg`, changing it as little as possible.
///
/// If even pure black/white cannot reach `min_ratio`, the closest achievable
/// color is returned. WCAG AA requires 4.5 for normal text and 3.0 for large
/// text.
pub const fn ensure_contrast(fg: Color, bg: Color, min_ratio: f32) -> Color {
    if contrast_ratio(fg, bg) >= min_ratio {
        return fg;
    }
    let lab = Oklab::from_color(fg);
    let need_darker = relative_luminance(bg) > relative_luminance(fg);
    let extreme = {
        let mut e = lab;
        e.l = if need_darker { 0.0 } else { 1.0 };
        e.to_color()
    };
    if contrast_ratio(extreme, bg) < min_ratio {
        return extreme;
    }
    // Binary-search the lightness closest to the original that meets the ratio.
    // Invariant: `lo`/`hi` bound the valid region from the invalid side.
    let mut lo = if need_darker { 0.0 } else { lab.l };
    let mut hi = if need_darker { lab.l } else { 1.0 };
    let mut best = extreme;
    let mut i = 0;
    while i < 24 {
        let mid = (lo + hi) * 0.5;
        let mut candidate = lab;
        candidate.l = mid;
        let candidate = candidate.to_color();
        if contrast_ratio(candidate, bg) >= min_ratio {
            best = candidate;
            if need_darker {
                hi = mid;
            } else {
                lo = mid;
            }
        } else if need_darker {
            lo = mid;
        } else {
            hi = mid;
        }
        i += 1;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, what: &str) {
        assert!((a - b).abs() < 1.0e-3, "{what}: {a} vs {b}");
    }

    #[test]
    fn alpha_ops() {
        let color = Color::from_rgb(1.0, 0.5, 0.0);
        assert_eq!(with_alpha(color, 0.25).a, 0.25);
        assert_eq!(scale_alpha(color, 0.5).a, 0.5);
        // Channels untouched
        assert_eq!(with_alpha(color, 0.25).g, 0.5);
    }

    #[test]
    fn hue_ops() {
        let red = Color::from_rgb(1.0, 0.0, 0.0);
        let green = with_hue(red, 120.0);
        close(green.r, 0.0, "r");
        close(green.g, 1.0, "g");
        close(green.b, 0.0, "b");

        let hsl = Hsl::from_color(with_hue(red, 210.0));
        close(hsl.h, 210.0, "h");
        close(hsl.s, 1.0, "s");
        close(hsl.l, 0.5, "l");
    }

    #[test]
    fn rotate_hue_full_turn_is_identity() {
        let color = Color::from_rgb(0.2, 0.6, 0.9);
        let rotated = rotate_hue(color, 360.0);
        assert!((rotated.r - color.r).abs() < 1.0e-5);
        assert!((rotated.g - color.g).abs() < 1.0e-5);
        assert!((rotated.b - color.b).abs() < 1.0e-5);
    }

    #[test]
    fn rotating_achromatic_is_noop() {
        let gray = Color::from_rgb(0.5, 0.5, 0.5);
        assert_eq!(rotate_hue(gray, 123.0), gray);
    }

    #[test]
    fn saturation_ops() {
        let color = Color::from_rgb(0.2, 0.6, 1.0);
        assert_eq!(with_saturation(color, 0.0), Color::from_rgb(0.6, 0.6, 0.6));
        close(desaturate(color, 0.5).b, 0.8, "b");
        close(saturate(desaturate(color, 0.2), 0.2).r, color.r, "r");
    }

    #[test]
    fn lightness_ops_clamp() {
        assert_eq!(darken(Color::BLACK, 0.2), Color::BLACK);
        assert_eq!(lighten(Color::WHITE, 0.2), Color::WHITE);
        close(lighten(Color::from_rgb(0.5, 0.5, 0.5), 0.25).r, 0.75, "r");
        close(darken(Color::from_rgb(0.5, 0.5, 0.5), 0.25).r, 0.25, "r");
    }

    #[test]
    fn luminance_ops() {
        let red = Color::from_rgb(1.0, 0.0, 0.0);
        let dimmed = with_luminance(red, 0.5);
        close(Oklab::from_color(dimmed).l, 0.5, "l");

        let brighter = adjust_luminance(red, 0.1);
        close(
            Oklab::from_color(brighter).l,
            Oklab::from_color(red).l + 0.1,
            "l",
        );
    }

    #[test]
    fn contrast_factor() {
        let color = Color::from_rgba8(100, 200, 255, 0.5);
        assert_eq!(with_contrast(color, 1.0), color);
        let flat = with_contrast(color, 0.0);
        assert!((flat.r - 0.5).abs() < 1.0e-6);
        assert!((flat.g - 0.5).abs() < 1.0e-6);
        assert!((flat.b - 0.5).abs() < 1.0e-6);
        assert_eq!(flat.a, 0.5);
        close(with_contrast(color, 2.0).r, 2.0 * color.r - 0.5, "r");
    }

    #[test]
    fn relative_luminance_extremes() {
        close(relative_luminance(Color::WHITE), 1.0, "white");
        close(relative_luminance(Color::BLACK), 0.0, "black");
    }

    #[test]
    fn contrast_ratio_wcag() {
        close(contrast_ratio(Color::BLACK, Color::WHITE), 21.0, "ratio");
        close(contrast_ratio(Color::WHITE, Color::BLACK), 21.0, "ratio");
        close(contrast_ratio(Color::WHITE, Color::WHITE), 1.0, "ratio");
    }

    #[test]
    fn ensure_contrast_reaches_target() {
        let background = Color::WHITE;
        let gray = Color::from_rgb8(128, 128, 128);
        assert!(contrast_ratio(gray, background) < 4.5);

        let text = ensure_contrast(gray, background, 4.5);
        assert!(contrast_ratio(text, background) >= 4.5);
        // It should have gotten darker, and only as much as needed.
        assert!(relative_luminance(text) < relative_luminance(gray));

        // Dark background: text should get lighter instead.
        let dark_gray = Color::from_rgb8(90, 90, 90);
        assert!(contrast_ratio(dark_gray, Color::BLACK) < 4.5);
        let text_on_black = ensure_contrast(dark_gray, Color::BLACK, 4.5);
        assert!(contrast_ratio(text_on_black, Color::BLACK) >= 4.5);
        assert!(relative_luminance(text_on_black) > relative_luminance(dark_gray));
    }

    #[test]
    fn ensure_contrast_returns_unchanged_when_met() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        assert_eq!(ensure_contrast(black, white, 4.5), black);
    }

    #[test]
    fn ensure_contrast_best_effort() {
        // An impossible target returns the extreme rather than looping forever.
        let gray = Color::from_rgb8(128, 128, 128);
        let result = ensure_contrast(gray, gray, 100.0);
        // Gray on itself: neither black nor white can reach 100:1; result is
        // driven to one of the extremes.
        let luminance = relative_luminance(result);
        assert!(
            luminance < 0.01 || luminance > 0.99,
            "luminance: {luminance}"
        );
    }

    #[test]
    fn const_context() {
        const TEXT: Color = ensure_contrast(Color::from_rgb8(128, 128, 128), Color::WHITE, 4.5);
        assert!(contrast_ratio(TEXT, Color::WHITE) >= 4.5);
    }
}
