//! Verifies that the macros and const fns are genuinely usable at compile
//! time, and that const evaluation agrees with runtime evaluation.

use iced_color_extension::{Color, IcedColorExt, hsl, oklab, rgb};

const ACCENT: Color = hsl!(210.0, 0.8, 0.5);
const GHOST: Color = hsl!(210.0, 0.8, 0.5, 0.4);
const DANGER: Color = rgb!(220, 60, 60);
const SUCCESS: Color = oklab!(0.72, -0.15, 0.1);
const TEXT: Color =
    iced_color_extension::ops::ensure_contrast(Color::from_rgb8(128, 128, 128), Color::WHITE, 4.5);

static OPAQUE: Color = rgb!(220, 60, 60, 0.5);

#[test]
fn const_items_evaluate() {
    assert_eq!(ACCENT.a, 1.0);
    assert_eq!(GHOST.a, 0.4);
    assert_eq!(OPAQUE.a, 0.5);
    for channel in [SUCCESS.r, SUCCESS.g, SUCCESS.b] {
        assert!((0.0..=1.0).contains(&channel));
    }
    assert!(iced_color_extension::ops::contrast_ratio(TEXT, Color::WHITE) >= 4.5);
}

#[test]
fn const_eval_matches_runtime_eval() {
    assert_eq!(ACCENT, hsl!(210.0, 0.8, 0.5));
    assert_eq!(GHOST, hsl!(210.0, 0.8, 0.5, 0.4));
    assert_eq!(DANGER, rgb!(220, 60, 60));
    assert_eq!(SUCCESS, oklab!(0.72, -0.15, 0.1));
    assert_eq!(
        TEXT,
        iced_color_extension::ops::ensure_contrast(
            Color::from_rgb8(128, 128, 128),
            Color::WHITE,
            4.5,
        )
    );
}

#[test]
fn rgb_macro_matches_iced_constructor() {
    assert_eq!(DANGER, Color::from_rgb8(220, 60, 60));
}

#[test]
fn derived_const_colors() {
    // Const fns compose: derive a palette at compile time.
    const HOVER: Color = iced_color_extension::ops::lighten(ACCENT, 0.1);
    const PRESSED: Color = iced_color_extension::ops::darken(ACCENT, 0.1);

    assert!(HOVER.to_hsl().l > ACCENT.to_hsl().l);
    assert!(PRESSED.to_hsl().l < ACCENT.to_hsl().l);
    assert_eq!(
        HOVER,
        iced_color_extension::ops::lighten(hsl!(210.0, 0.8, 0.5), 0.1)
    );
}
