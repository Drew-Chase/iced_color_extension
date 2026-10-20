//! Internal `const fn` floating-point math.
//!
//! The standard library's transcendental helpers ([`f32::sqrt`], [`f32::cbrt`],
//! [`f32::powf`], ...) are not callable in `const` contexts on stable Rust, so
//! this module provides Newton-Raphson based replacements that are. Every
//! function converges to within a couple of ULPs of the libm result, which is
//! orders of magnitude below any perceptible color difference.

/// Absolute value of `x`.
#[inline]
pub(crate) const fn abs(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7FFF_FFFF)
}

/// Returns `true` if `x` is neither infinite nor NaN.
#[inline]
pub(crate) const fn is_finite(x: f32) -> bool {
    (x.to_bits() & 0x7F80_0000) != 0x7F80_0000
}

/// Clamps `x` into `0.0..=1.0`.
#[inline]
pub(crate) const fn clamp01(x: f32) -> f32 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Square root via Newton-Raphson (~1 ULP).
#[inline]
pub(crate) const fn sqrt(x: f32) -> f32 {
    if x == 0.0 {
        return x;
    }
    if x < 0.0 {
        return f32::NAN;
    }
    // Seed from the exponent: sqrt halves it. (0.5 * 127 * 2^23)
    let mut y = f32::from_bits((x.to_bits() >> 1) + 0x1FC0_0000);
    let mut i = 0;
    while i < 4 {
        y = 0.5 * (y + x / y);
        i += 1;
    }
    y
}

/// Cube root via Newton-Raphson (~1 ULP). Negative inputs yield negative roots.
#[inline]
pub(crate) const fn cbrt(x: f32) -> f32 {
    if x == 0.0 {
        return x;
    }
    let negative = x < 0.0;
    let ax = if negative { -x } else { x };
    // Seed from the exponent: cbrt divides it by three. (127 * 2/3 * 2^23)
    let mut y = f32::from_bits(ax.to_bits() / 3 + 0x2A55_5555);
    let mut i = 0;
    while i < 5 {
        y = (2.0 * y + ax / (y * y)) / 3.0;
        i += 1;
    }
    if negative { -y } else { y }
}

/// Fifth root via Newton-Raphson (~1 ULP). Negative inputs yield negative roots.
#[inline]
pub(crate) const fn fifth_root(x: f32) -> f32 {
    if x == 0.0 {
        return 0.0;
    }
    let negative = x < 0.0;
    let ax = if negative { -x } else { x };
    // Seed from the exponent: fifth root divides it by five. (127 * 4/5 * 2^23)
    let mut y = f32::from_bits(ax.to_bits() / 5 + 0x32CC_CCCD);
    let mut i = 0;
    while i < 6 {
        let y4 = y * y * y * y;
        y = (4.0 * y + ax / y4) / 5.0;
        i += 1;
    }
    if negative { -y } else { y }
}

/// `x^2.4`, the exponent of the sRGB electro-optical transfer function.
#[inline]
pub(crate) const fn pow_2_4(x: f32) -> f32 {
    // x^2.4 = x^2 * x^0.4 = x^2 * (x^2)^(1/5)
    let x2 = x * x;
    x2 * fifth_root(x2)
}

/// `x^(5/12)`, the exponent of the inverse sRGB transfer function.
#[inline]
pub(crate) const fn pow_5_12(x: f32) -> f32 {
    // x^(5/12) = x^(1/3) * x^(1/12) = cbrt(x) * sqrt(sqrt(cbrt(x)))
    let t = cbrt(x);
    t * sqrt(sqrt(t))
}

/// sRGB electro-optical transfer function: encoded `0.0..=1.0` -> linear light.
#[inline]
pub(crate) const fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.040_45 {
        x / 12.92
    } else {
        pow_2_4((x + 0.055) / 1.055)
    }
}

/// Inverse sRGB transfer function: linear light -> encoded `0.0..=1.0`.
#[inline]
pub(crate) const fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * pow_5_12(x) - 0.055
    }
}

/// Wraps a hue in degrees into `0.0..360.0`.
///
/// Non-finite inputs are returned unchanged. Absurdly large finite inputs
/// (beyond what the saturating bulk reduction can handle) fall back to `0.0`.
#[inline]
pub(crate) const fn normalize_hue(h: f32) -> f32 {
    if !is_finite(h) {
        return h;
    }
    // Bulk-reduce whole turns. The saturating cast keeps huge inputs finite.
    let turns = (h / 360.0) as i32;
    let mut h = h - turns as f32 * 360.0;
    // Float rounding can leave the remainder slightly out of range.
    let mut i = 0;
    while h < 0.0 && i < 4 {
        h += 360.0;
        i += 1;
    }
    while h >= 360.0 && i < 12 {
        h -= 360.0;
        i += 1;
    }
    if h < 0.0 || h >= 360.0 {
        return 0.0;
    }
    h
}

/// Rounds `x` to `1/factor` for `Display` output (runtime only).
pub(crate) fn round_for_display(x: f32, factor: f32) -> f32 {
    let r = (x * factor).round() / factor;
    if r == 0.0 { 0.0 } else { r }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f32, b: f32, rel: f32) {
        assert!(
            (a - b).abs() <= rel * b.abs().max(1.0),
            "expected {a} to be close to {b} (rel {rel})"
        );
    }

    #[test]
    fn sqrt_matches_std() {
        let mut x = 1.0e-6f32;
        let mut n = 0;
        while n < 400 {
            assert_close(sqrt(x), x.sqrt(), 1.0e-5);
            x *= 1.1;
            n += 1;
        }
        assert_close(sqrt(0.003_130_8), 0.003_130_8f32.sqrt(), 1.0e-5);
        assert_eq!(sqrt(0.0), 0.0);
        assert!(sqrt(-1.0).is_nan());
    }

    #[test]
    fn cbrt_matches_std() {
        let mut x = 1.0e-9f32;
        let mut n = 0;
        while n < 300 {
            assert_close(cbrt(x), x.cbrt(), 1.0e-5);
            assert_close(cbrt(-x), -x.cbrt(), 1.0e-5);
            x *= 1.3;
            n += 1;
        }
        assert_close(cbrt(8.0), 2.0, 1.0e-6);
        assert_eq!(cbrt(0.0), 0.0);
    }

    #[test]
    fn fifth_root_matches_std() {
        let mut x = 1.0e-9f32;
        let mut n = 0;
        while n < 300 {
            assert_close(fifth_root(x), x.powf(0.2), 1.0e-5);
            x *= 1.3;
            n += 1;
        }
        assert_close(fifth_root(32.0), 2.0, 1.0e-6);
    }

    #[test]
    fn transfer_functions_match_std() {
        let mut x = 0.0f32;
        let mut n = 0;
        while n < 1000 {
            let expected = if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            };
            assert_close(srgb_to_linear(x), expected, 1.0e-5);
            x += 0.001;
            n += 1;
        }
        // Round trip through both transfer functions.
        x = 0.001;
        n = 0;
        while n < 1000 {
            assert_close(linear_to_srgb(srgb_to_linear(x)), x, 1.0e-4);
            x += 0.001;
            n += 1;
        }
    }

    #[test]
    fn pow_helpers_match_std() {
        let mut x = 0.05f32;
        let mut n = 0;
        while n < 200 {
            assert_close(pow_2_4(x), x.powf(2.4), 1.0e-5);
            assert_close(pow_5_12(x), x.powf(5.0 / 12.0), 1.0e-5);
            x *= 1.05;
            n += 1;
        }
    }

    #[test]
    fn hue_normalization() {
        assert_eq!(normalize_hue(0.0), 0.0);
        assert_eq!(normalize_hue(360.0), 0.0);
        assert_eq!(normalize_hue(-90.0), 270.0);
        assert_eq!(normalize_hue(725.0), 5.0);
        assert_eq!(normalize_hue(-720.0), 0.0);
        assert_eq!(normalize_hue(1234.5), 154.5);
        assert!(normalize_hue(f32::NAN).is_nan());
        assert!(normalize_hue(f32::INFINITY).is_infinite());
    }

    #[test]
    fn misc_helpers() {
        assert_eq!(abs(-3.5), 3.5);
        assert_eq!(abs(3.5), 3.5);
        assert_eq!(clamp01(-0.1), 0.0);
        assert_eq!(clamp01(1.1), 1.0);
        assert_eq!(clamp01(0.25), 0.25);
        assert!(is_finite(1.0));
        assert!(!is_finite(f32::NEG_INFINITY));
    }
}
