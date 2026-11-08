//! Perceptual colour science for the theme (STORY-0113, `docs/specs/ui-colour-and-type.md`): Oklab and OKLCH
//! (Björn Ottosson, 2020) for choosing colours by how light they look, and the APCA lightness contrast `Lc`
//! (the WCAG 3 draft, APCA-W3 0.0.98G-4g) for checking that text on a background can be read. Pure, no crate:
//! two 3 x 3 matrices, a cube root and a handful of powers (build-vs-buy: the crates offer the same maths and a
//! dependency).

use eframe::egui::Color32;

/// A colour in OKLCH: perceptual lightness `l` (0 black to 1 white), chroma `c` (0 grey, about 0.37 at most
/// in sRGB) and hue angle `h` in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    /// Lightness, 0 to 1.
    pub l: f32,
    /// Chroma, 0 and up.
    pub c: f32,
    /// Hue, degrees.
    pub h: f32,
}

/// A colour in OKLCH (a shorter way to write [`Oklch`]).
pub const fn lch(l: f32, c: f32, h: f32) -> Oklch {
    Oklch { l, c, h }
}

/// An sRGB channel (0 to 255) as linear light (0 to 1), by the sRGB transfer function.
pub fn to_linear(v: u8) -> f64 {
    let v = f64::from(v) / 255.0;
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(v: f64) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// Linear sRGB to Oklab `[L, a, b]`.
pub fn linear_to_oklab([r, g, b]: [f64; 3]) -> [f64; 3] {
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// Oklab `[L, a, b]` to linear sRGB (may fall outside 0 to 1 when the colour is outside sRGB).
pub fn oklab_to_linear([ll, a, b]: [f64; 3]) -> [f64; 3] {
    let l = (ll + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (ll - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (ll - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ]
}

/// The Oklab coordinates of an sRGB colour.
pub fn rgb_to_oklab(c: Color32) -> [f64; 3] {
    linear_to_oklab([to_linear(c.r()), to_linear(c.g()), to_linear(c.b())])
}

/// The OKLCH coordinates of an sRGB colour (`h` is 0 for greys).
pub fn rgb_to_oklch(c: Color32) -> Oklch {
    let [l, a, b] = rgb_to_oklab(c);
    let chroma = a.hypot(b);
    let hue = if chroma < 1e-6 {
        0.0
    } else {
        b.atan2(a).to_degrees().rem_euclid(360.0)
    };
    lch(l as f32, chroma as f32, hue as f32)
}

fn lab_of(o: Oklch, c: f64) -> [f64; 3] {
    let h = f64::from(o.h).to_radians();
    [f64::from(o.l), c * h.cos(), c * h.sin()]
}

fn fits(rgb: [f64; 3]) -> bool {
    rgb.iter().all(|v| (-1e-4..=1.0 + 1e-4).contains(v))
}

/// Whether the OKLCH colour can be shown in sRGB as it is.
pub fn in_gamut(o: Oklch) -> bool {
    fits(oklab_to_linear(lab_of(o, f64::from(o.c))))
}

/// The largest chroma up to `limit` that sRGB can show at this lightness and hue. The gamut's slice is not convex
/// near its corners, so it steps down from `limit` until the colour fits, then bisects the last step (to 1e-5).
pub fn fit_chroma(l: f32, h: f32, limit: f32) -> f32 {
    const STEP: f32 = 0.005;
    let fits = |c: f32| in_gamut(lch(l, c, h));
    let mut hi = limit.max(0.0);
    if fits(hi) {
        return hi;
    }
    let mut lo = (hi - STEP).max(0.0);
    while lo > 0.0 && !fits(lo) {
        hi = lo;
        lo = (lo - STEP).max(0.0);
    }
    while hi - lo > 1e-5 {
        let mid = (lo + hi) / 2.0;
        if fits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The largest chroma sRGB can show at this lightness and hue.
pub fn max_chroma(l: f32, h: f32) -> f32 {
    fit_chroma(l, h, 0.4)
}

/// An OKLCH colour in sRGB. A colour outside sRGB keeps its lightness and hue and loses chroma until it fits
/// (the gamut mapping CSS Color 4 describes, without its just-noticeable-difference shortcut).
pub fn oklch_to_rgb(o: Oklch) -> Color32 {
    let c = f64::from(fit_chroma(o.l, o.h, o.c));
    let [r, g, b] = oklab_to_linear(lab_of(o, c));
    Color32::from_rgb(from_linear(r), from_linear(g), from_linear(b))
}

/// `a` and `b` mixed in Oklab (`share` of `b`, 0 to 1), so a tint changes lightness evenly whatever its hue.
pub fn mix_oklab(a: Color32, b: Color32, share: f32) -> Color32 {
    let (x, y, t) = (
        rgb_to_oklab(a),
        rgb_to_oklab(b),
        f64::from(share.clamp(0.0, 1.0)),
    );
    let m = [0, 1, 2].map(|i| x[i] + (y[i] - x[i]) * t);
    let [r, g, b] = oklab_to_linear(m);
    Color32::from_rgb(from_linear(r), from_linear(g), from_linear(b))
}

/// APCA's screen luminance of a colour: a plain 2.4 power per channel, as APCA-W3 defines it.
pub fn apca_y(c: Color32) -> f64 {
    let ch = |v: u8| (f64::from(v) / 255.0).powf(2.4);
    0.212_672_9 * ch(c.r()) + 0.715_152_2 * ch(c.g()) + 0.072_175 * ch(c.b())
}

fn soft_clamp(y: f64) -> f64 {
    const BLACK_THRESHOLD: f64 = 0.022;
    if y < BLACK_THRESHOLD {
        y + (BLACK_THRESHOLD - y).powf(1.414)
    } else {
        y
    }
}

/// The APCA lightness contrast `Lc` of `text` on `bg`: positive for dark text on a light background, negative
/// for light text on a dark one, about -108 to 106; 0 when the contrast is too low to count (|Lc| under 10).
/// Read its size: 90 is preferred for body text, 75 the least for body text, 60 for other content text, 45 for
/// large text and icons, 15 the least for a divider that should be seen.
pub fn apca_lc(text: Color32, bg: Color32) -> f32 {
    let (yt, yb) = (soft_clamp(apca_y(text)), soft_clamp(apca_y(bg)));
    if (yb - yt).abs() < 0.0005 {
        return 0.0;
    }
    let out = if yb > yt {
        let s = (yb.powf(0.56) - yt.powf(0.57)) * 1.14;
        if s < 0.1 { 0.0 } else { s - 0.027 }
    } else {
        let s = (yb.powf(0.65) - yt.powf(0.62)) * 1.14;
        if s > -0.1 { 0.0 } else { s + 0.027 }
    };
    (out * 100.0) as f32
}

/// The size of the APCA contrast, whatever the polarity.
pub fn apca_abs(text: Color32, bg: Color32) -> f32 {
    apca_lc(text, bg).abs()
}
