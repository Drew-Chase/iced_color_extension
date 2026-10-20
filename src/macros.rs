//! Compile-time color construction macros.
//!
//! All macros expand to calls to `const fn`s, so they can be used in `const`
//! items, `static`s, and constant generics:
//!
//! ```
//! use iced_core::Color;
//! use iced_color_extension::{hsl, oklab, rgb};
//!
//! const ACCENT: Color = hsl!(210.0, 0.8, 0.5);
//! const DANGER: Color = rgb!(220, 60, 60);
//! const SUCCESS: Color = oklab!(0.72, -0.15, 0.1);
//! ```

/// Creates an iced [`Color`](iced_core::Color) from HSL components.
///
/// - `h` — hue in degrees (wrapped into `0.0..360.0`)
/// - `s` — saturation, `0.0..=1.0`
/// - `l` — lightness, `0.0..=1.0`
/// - `a` (optional) — alpha, `0.0..=1.0` (defaults to `1.0`)
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::hsl;
///
/// const ACCENT: Color = hsl!(210.0, 0.8, 0.5);
/// const GHOST: Color = hsl!(210.0, 0.8, 0.5, 0.5);
///
/// assert_eq!(ACCENT.a, 1.0);
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! hsl {
    ($h:expr, $s:expr, $l:expr $(,)?) => {
        $crate::Hsl::new($h, $s, $l).to_color()
    };
    ($h:expr, $s:expr, $l:expr, $a:expr $(,)?) => {
        $crate::Hsl::new($h, $s, $l).with_alpha($a).to_color()
    };
}

/// Creates an iced [`Color`](iced_core::Color) from sRGB channels in `0..255`,
/// like CSS `rgb()`.
///
/// - `r`, `g`, `b` — channels, `0..=255`
/// - `a` (optional) — alpha, `0.0..=1.0` (defaults to `1.0`)
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::rgb;
///
/// const DANGER: Color = rgb!(220, 60, 60);
/// const GHOST: Color = rgb!(220, 60, 60, 0.5);
///
/// assert_eq!(DANGER, Color::from_rgb8(220, 60, 60));
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        $crate::Rgb::from_rgb8($r, $g, $b).to_color()
    };
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => {
        $crate::Rgb::from_rgb8($r, $g, $b).with_alpha($a).to_color()
    };
}

/// Creates an iced [`Color`](iced_core::Color) from OKLAB components.
///
/// - `l` — perceptual lightness, `0.0..=1.0`
/// - `a` — green–red opponent axis
/// - `b` — blue–yellow opponent axis
/// - `alpha` (optional) — opacity, `0.0..=1.0` (defaults to `1.0`)
///
/// Colors outside the sRGB gamut are mapped back by reducing chroma while
/// preserving lightness.
///
/// ```
/// use iced_core::Color;
/// use iced_color_extension::oklab;
///
/// const SUCCESS: Color = oklab!(0.72, -0.15, 0.1);
/// const GHOST: Color = oklab!(0.72, -0.15, 0.1, 0.5);
///
/// assert_eq!(SUCCESS.a, 1.0);
/// assert_eq!(GHOST.a, 0.5);
/// ```
#[macro_export]
macro_rules! oklab {
    ($l:expr, $a:expr, $b:expr $(,)?) => {
        $crate::Oklab::new($l, $a, $b).to_color()
    };
    ($l:expr, $a:expr, $b:expr, $alpha:expr $(,)?) => {
        $crate::Oklab::new($l, $a, $b).with_alpha($alpha).to_color()
    };
}
