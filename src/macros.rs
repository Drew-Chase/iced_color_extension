//! Compile-time color construction macros.
//!
//! All macros expand to calls to `const fn`s, so they can be used in `const`
//! items, `static`s, and constant generics. Every component accepts any
//! numeric type (via `as f32`), plus CSS-style percentage literals:
//!
//! ```
//! use iced_core::Color;
//! use iced_color_extension::{hsl, oklab, rgb};
//!
//! const ACCENT: Color = hsl!(147, 100%, 57%);
//! const DANGER: Color = rgb!(220, 60, 60);
//! const GHOST: Color = rgb!(100%, 20%, 20%, 50%);
//! const SUCCESS: Color = oklab!(50%, 0.1, 0.1);
//! ```
//!
//! Percentage forms require *literal* numbers (`50%`, `12.5%`); computed
//! expressions should pass fractions instead (`0.5`).

/// Creates an iced [`Color`](iced_core::Color) from HSL components.
///
/// - `h` — hue in degrees, any numeric expression (wrapped into `0.0..360.0`)
/// - `s` — saturation: any number (`0.0..=1.0`) or a percentage literal
///   (`100%` = `1.0`)
/// - `l` — lightness: same forms as `s`
/// - `a` (optional) — alpha: same forms as `s` (defaults to `1.0`)
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::hsl;
///
/// const ACCENT: Color = hsl!(147, 100%, 57%);
/// const GHOST: Color = hsl!(147, 100%, 57%, 50%);
///
/// // All of these are equivalent:
/// assert_eq!(hsl!(147, 100%, 57%), hsl!(147, 1, 0.57));
/// assert_eq!(hsl!(147, 100%, 57%), hsl!(147.0, 1.0, 0.57));
/// assert_eq!(hsl!(147, 100%, 57%), hsl!(147, 1, 57%));
///
/// assert_eq!(ACCENT.a, 1.0);
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! hsl {
    ($h:expr, $s:literal %, $l:literal %, $a:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32 / 100.0)
            .with_alpha(($a) as f32 / 100.0)
            .to_color()
    };
    ($h:expr, $s:literal %, $l:literal %, $a:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32 / 100.0)
            .with_alpha(($a) as f32)
            .to_color()
    };
    ($h:expr, $s:literal %, $l:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32 / 100.0).to_color()
    };
    ($h:expr, $s:literal %, $l:expr, $a:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32)
            .with_alpha(($a) as f32 / 100.0)
            .to_color()
    };
    ($h:expr, $s:literal %, $l:expr, $a:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32)
            .with_alpha(($a) as f32)
            .to_color()
    };
    ($h:expr, $s:literal %, $l:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32 / 100.0, ($l) as f32).to_color()
    };
    ($h:expr, $s:expr, $l:literal %, $a:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32 / 100.0)
            .with_alpha(($a) as f32 / 100.0)
            .to_color()
    };
    ($h:expr, $s:expr, $l:literal %, $a:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32 / 100.0)
            .with_alpha(($a) as f32)
            .to_color()
    };
    ($h:expr, $s:expr, $l:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32 / 100.0).to_color()
    };
    ($h:expr, $s:expr, $l:expr, $a:literal % $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32)
            .with_alpha(($a) as f32 / 100.0)
            .to_color()
    };
    ($h:expr, $s:expr, $l:expr, $a:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32)
            .with_alpha(($a) as f32)
            .to_color()
    };
    ($h:expr, $s:expr, $l:expr $(,)?) => {
        $crate::Hsl::new(($h) as f32, ($s) as f32, ($l) as f32).to_color()
    };
}

/// Creates an iced [`Color`](iced_core::Color) from sRGB channels, like CSS
/// `rgb()`.
///
/// - `r`, `g`, `b` — channels on a `0..=255` scale (any numeric type) or
///   percentage literals (`100%` = `1.0`); percentages apply to all three
///   channels
/// - `a` (optional) — alpha: any number (`0.0..=1.0`) or a percentage literal
///   (defaults to `1.0`)
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::rgb;
///
/// const DANGER: Color = rgb!(220, 60, 60);
/// const GHOST: Color = rgb!(100%, 20%, 20%, 50%);
///
/// assert_eq!(rgb!(100%, 20%, 20%), rgb!(255, 51, 51));
/// assert_eq!(DANGER, Color::from_rgb8(220, 60, 60));
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! rgb {
    ($r:literal %, $g:literal %, $b:literal %, $a:literal % $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 100.0,
            ($g) as f32 / 100.0,
            ($b) as f32 / 100.0,
        )
        .with_alpha(($a) as f32 / 100.0)
        .to_color()
    };
    ($r:literal %, $g:literal %, $b:literal %, $a:expr $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 100.0,
            ($g) as f32 / 100.0,
            ($b) as f32 / 100.0,
        )
        .with_alpha(($a) as f32)
        .to_color()
    };
    ($r:literal %, $g:literal %, $b:literal % $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 100.0,
            ($g) as f32 / 100.0,
            ($b) as f32 / 100.0,
        )
        .to_color()
    };
    ($r:expr, $g:expr, $b:expr, $a:literal % $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 255.0,
            ($g) as f32 / 255.0,
            ($b) as f32 / 255.0,
        )
        .with_alpha(($a) as f32 / 100.0)
        .to_color()
    };
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 255.0,
            ($g) as f32 / 255.0,
            ($b) as f32 / 255.0,
        )
        .with_alpha(($a) as f32)
        .to_color()
    };
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        $crate::Rgb::new(
            ($r) as f32 / 255.0,
            ($g) as f32 / 255.0,
            ($b) as f32 / 255.0,
        )
        .to_color()
    };
}

/// Creates an iced [`Color`](iced_core::Color) from OKLAB components.
///
/// - `l` — perceptual lightness: any number (`0.0..=1.0`) or a percentage
///   literal (`50%` = `0.5`)
/// - `a` — green–red opponent axis (any numeric type)
/// - `b` — blue–yellow opponent axis (any numeric type)
/// - `alpha` (optional) — opacity: any number or a percentage literal
///   (defaults to `1.0`)
///
/// Colors outside the sRGB gamut are mapped back by reducing chroma while
/// preserving lightness.
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::oklab;
///
/// const SUCCESS: Color = oklab!(72%, -0.15, 0.1);
/// const GHOST: Color = oklab!(0.72, -0.15, 0.1, 50%);
///
/// assert_eq!(oklab!(72%, -0.15, 0.1), oklab!(0.72, -0.15, 0.1));
/// assert_eq!(SUCCESS.a, 1.0);
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! oklab {
    ($l:literal %, $a:expr, $b:expr, $alpha:literal % $(,)?) => {
        $crate::Oklab::new(($l) as f32 / 100.0, ($a) as f32, ($b) as f32)
            .with_alpha(($alpha) as f32 / 100.0)
            .to_color()
    };
    ($l:literal %, $a:expr, $b:expr, $alpha:expr $(,)?) => {
        $crate::Oklab::new(($l) as f32 / 100.0, ($a) as f32, ($b) as f32)
            .with_alpha(($alpha) as f32)
            .to_color()
    };
    ($l:literal %, $a:expr, $b:expr $(,)?) => {
        $crate::Oklab::new(($l) as f32 / 100.0, ($a) as f32, ($b) as f32).to_color()
    };
    ($l:expr, $a:expr, $b:expr, $alpha:literal % $(,)?) => {
        $crate::Oklab::new(($l) as f32, ($a) as f32, ($b) as f32)
            .with_alpha(($alpha) as f32 / 100.0)
            .to_color()
    };
    ($l:expr, $a:expr, $b:expr, $alpha:expr $(,)?) => {
        $crate::Oklab::new(($l) as f32, ($a) as f32, ($b) as f32)
            .with_alpha(($alpha) as f32)
            .to_color()
    };
    ($l:expr, $a:expr, $b:expr $(,)?) => {
        $crate::Oklab::new(($l) as f32, ($a) as f32, ($b) as f32).to_color()
    };
}

#[cfg(test)]
mod tests {
    use std::format;

    use crate::{Color, Hsl, Oklab};

    #[test]
    fn hsl_forms_are_equivalent() {
        let expected = Hsl::new(147.0, 1.0, 0.57).to_color();
        assert_eq!(hsl!(147, 100%, 57%), expected);
        assert_eq!(hsl!(147, 1, 0.57), expected);
        assert_eq!(hsl!(147.0, 1.0, 0.57), expected);
        assert_eq!(hsl!(147, 100%, 0.57), expected);
        assert_eq!(hsl!(147, 1, 57%), expected);
        // Trailing commas
        assert_eq!(hsl!(147, 100%, 57%,), expected);
        assert_eq!(hsl!(147, 1, 0.57,), expected);
    }

    #[test]
    fn hsl_alpha_forms() {
        assert_eq!(hsl!(147, 1, 0.57, 50%).a, 0.5);
        assert_eq!(hsl!(147, 100%, 57%, 100%), hsl!(147, 1, 0.57));
        assert_eq!(hsl!(147, 1, 0.57, 0.5).a, 0.5);
        assert_eq!(hsl!(147, 1, 0.57, 50%,).a, 0.5);
        assert_eq!(hsl!(147, 100%, 0.57, 50%).a, 0.5);
        assert_eq!(hsl!(147, 1, 57%, 0.5).a, 0.5);
    }

    #[test]
    fn hsl_wraps_negative_integer_hue() {
        assert_eq!(hsl!(-90, 100%, 50%), hsl!(270, 1, 0.5));
    }

    #[test]
    fn hsl_accepts_any_numeric_type() {
        let h: u16 = 210;
        let s: f64 = 0.8;
        let l: i32 = 50;
        assert_eq!(hsl!(h, s, l as f32 / 100.0), hsl!(210.0, 0.8, 0.5));
    }

    #[test]
    fn rgb_forms() {
        assert_eq!(rgb!(100%, 20%, 20%), rgb!(255, 51, 51));
        assert_eq!(rgb!(100%, 20%, 20%), Color::from_rgb8(255, 51, 51));
        assert_eq!(rgb!(220, 60, 60), Color::from_rgb8(220, 60, 60));
        assert_eq!(rgb!(220.0, 60, 60), Color::from_rgb8(220, 60, 60));
        assert_eq!(rgb!(100%, 20%, 20%, 50%).a, 0.5);
        assert_eq!(rgb!(220, 60, 60, 50%).a, 0.5);
        assert_eq!(rgb!(220, 60, 60, 0.5).a, 0.5);
        assert_eq!(rgb!(220, 60, 60,), Color::from_rgb8(220, 60, 60));
    }

    #[test]
    fn oklab_forms() {
        let expected = Oklab::new(0.72, -0.15, 0.1).to_color();
        assert_eq!(oklab!(72%, -0.15, 0.1), expected);
        assert_eq!(oklab!(0.72, -0.15, 0.1), expected);
        assert_eq!(oklab!(0.72, -0.15, 0.1, 50%).a, 0.5);
        assert_eq!(oklab!(72%, -0.15, 0.1, 50%).a, 0.5);
        assert_eq!(oklab!(0.72, -0.15, 0.1,), expected);
    }

    #[test]
    fn percent_of_float_channels() {
        // Fractional percentages work too.
        assert_eq!(hsl!(147, 50%, 57%), hsl!(147, 0.5, 0.57));
        assert_eq!(rgb!(12.5%, 100%, 0%).into_rgba8(), [32, 255, 0, 255]);
    }

    #[test]
    fn display_round_trip_with_macros() {
        use crate::IcedColorExt;

        let color = hsl!(147, 100%, 57%);
        let hsl = color.to_hsl();
        assert_eq!(format!("{hsl}"), "hsl(147 100% 57%)");
    }
}
