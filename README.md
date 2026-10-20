# iced_color_extension

Extended color formats and manipulation for [iced](https://iced.rs): **HSL**, **RGB** and **OKLAB**, with as much as possible computed at compile time.

Iced's `Color` only supports hex strings and a handful of runtime helpers. This crate adds:

- **New formats** — `hsl!`, `rgb!`, `oklab!` macros and `Hsl` / `Rgb` / `Oklab` structs with conversions to/from `iced::Color`
- **Perceptual manipulation** — lightness, hue, saturation, contrast (CSS-style and WCAG) and alpha
- **Compile-time support** — every conversion and manipulation is a `const fn` (including OKLAB, via const Newton-iteration `cbrt`/`x^2.4`), so palettes can be built in `const` items
- **CSS parsing/formatting** — parse `rgb()`, `hsl()`, `oklab()` and hex strings (iced only parses hex); CSS-flavored `Display` for every struct

Works with `iced` or just `iced_core` (version 0.14) — `iced::Color` is a re-export of `iced_core::Color`.

## Usage

```toml
[dependencies]
iced_color_extension = "0.1"
```

```rust
use iced_core::Color;
use iced_color_extension::{hsl, rgb, oklab, IcedColorExt};

// Compile-time colors — every macro and conversion is a `const fn`:
const ACCENT: Color = hsl!(210.0, 0.8, 0.5);
const DANGER: Color = rgb!(220, 60, 60);
const SUCCESS: Color = oklab!(0.72, -0.15, 0.1);

// Runtime manipulation through the extension trait:
let text = DANGER.lighten(0.15).ensure_contrast(ACCENT, 4.5);
let hue = text.to_hsl().h;
let muted = text.desaturate(0.3).scale_alpha(0.8);
```

### Compile-time (const) usage

Trait methods cannot be `const fn` on stable Rust, so the const entry points are the macros, the conversion structs, and the free functions in [`ops`](https://docs.rs/iced_color_extension/latest/iced_color_extension/ops/):

```rust
use iced_core::Color;
use iced_color_extension::{hsl, ops};

const MUTED_TEXT: Color = ops::lighten(ops::with_saturation(hsl!(210.0, 0.8, 0.5), 0.2), 0.75);
const READABLE: Color = ops::ensure_contrast(MUTED_TEXT, Color::WHITE, 4.5);
```

## Semantics

| Operation | Basis |
|---|---|
| `lighten` / `darken` / `with_lightness` | HSL lightness |
| `with_luminance` / `adjust_luminance` | OKLAB perceptual lightness (gamut-mapped) |
| `relative_luminance` / `contrast_ratio` / `ensure_contrast` | WCAG 2.1 |
| `with_contrast` | CSS `filter: contrast()` |
| `with_hue` / `rotate_hue` / `with_saturation` | HSL |

OKLAB colors outside the sRGB gamut are mapped back by reducing chroma while preserving lightness and hue.

## MSRV

Rust 1.88 (matching iced 0.14).

## License

MIT
