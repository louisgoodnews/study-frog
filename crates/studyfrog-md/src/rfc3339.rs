//! RFC 3339 timestamps to and from [`UnixMs`], years 0000 to 9999. Calendar arithmetic is shared
//! with `studyfrog_core::civil`, so `md` needs no date crate.

use studyfrog_core::civil::{civil_from_days, days_from_civil, days_in_month};
use studyfrog_core::time::UnixMs;

const MS_PER_DAY: i64 = 86_400_000;

/// Read `len` ASCII digits at `*pos`, advancing it.
fn number(bytes: &[u8], pos: &mut usize, len: usize) -> Option<i64> {
    let slice = bytes.get(*pos..*pos + len)?;
    *pos += len;
    slice.iter().try_fold(0_i64, |acc, b| {
        b.is_ascii_digit().then(|| acc * 10 + i64::from(b - b'0'))
    })
}

fn expect(bytes: &[u8], pos: &mut usize, allowed: &[u8]) -> Option<u8> {
    let b = *bytes.get(*pos)?;
    allowed.contains(&b).then(|| *pos += 1)?;
    Some(b)
}

/// Milliseconds from an optional `.fraction` (extra digits truncated).
fn fraction_ms(bytes: &[u8], pos: &mut usize) -> Option<i64> {
    if bytes.get(*pos) != Some(&b'.') {
        return Some(0);
    }
    *pos += 1;
    let start = *pos;
    while bytes.get(*pos).is_some_and(u8::is_ascii_digit) {
        *pos += 1;
    }
    let digits = &bytes[start..*pos];
    (!digits.is_empty()).then(|| {
        (0..3).fold(0, |acc, i| {
            acc * 10 + i64::from(digits.get(i).map_or(0, |d| d - b'0'))
        })
    })
}

/// Offset in milliseconds from `Z` or `+HH:MM` / `-HH:MM`.
fn offset_ms(bytes: &[u8], pos: &mut usize) -> Option<i64> {
    let sign = match expect(bytes, pos, b"Zz+-")? {
        b'-' => -1,
        b'+' => 1,
        _ => return Some(0),
    };
    let (h, m) = (number(bytes, pos, 2)?, {
        expect(bytes, pos, b":")?;
        number(bytes, pos, 2)?
    });
    (h < 24 && m < 60).then_some(sign * (h * 60 + m) * 60_000)
}

/// Parse `YYYY-MM-DDTHH:MM:SS[.fff](Z|+HH:MM|-HH:MM)`. A space may replace `T`.
pub fn parse_rfc3339(text: &str) -> Option<UnixMs> {
    let b = text.as_bytes();
    let mut p = 0;
    let y = number(b, &mut p, 4)?;
    expect(b, &mut p, b"-")?;
    let mo = number(b, &mut p, 2)?;
    expect(b, &mut p, b"-")?;
    let d = number(b, &mut p, 2)?;
    expect(b, &mut p, b"Tt ")?;
    let (h, mi) = (number(b, &mut p, 2)?, {
        expect(b, &mut p, b":")?;
        number(b, &mut p, 2)?
    });
    expect(b, &mut p, b":")?;
    let s = number(b, &mut p, 2)?;
    let frac = fraction_ms(b, &mut p)?;
    let off = offset_ms(b, &mut p)?;
    let valid = (1..=12).contains(&mo)
        && (1..=days_in_month(y, mo)).contains(&d)
        && h < 24
        && mi < 60
        && s < 60;
    (valid && p == b.len()).then(|| {
        let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s;
        UnixMs(secs * 1000 + frac - off)
    })
}

/// Format as `YYYY-MM-DDTHH:MM:SS.mmmZ`; `None` outside years 0000 to 9999.
pub fn format_rfc3339(at: UnixMs) -> Option<String> {
    let days = at.0.div_euclid(MS_PER_DAY);
    let ms = at.0.rem_euclid(MS_PER_DAY);
    let (y, m, d) = civil_from_days(days);
    (0..=9999).contains(&y).then(|| {
        let (s, milli) = (ms / 1000, ms % 1000);
        let (h, mi, sec) = (s / 3600, s % 3600 / 60, s % 60);
        format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{sec:02}.{milli:03}Z")
    })
}
