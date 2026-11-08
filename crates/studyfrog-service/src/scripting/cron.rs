//! Five-field cron schedules (`minute hour day-of-month month day-of-week`), matched against local
//! time. Written in-house (P10): lists, ranges, steps, names and the `@daily`-style shorthands are
//! all a script runner needs; no seconds, years or iterators.

use studyfrog_core::civil::{LocalTime, weekday, weekday_index};

/// A parsed schedule: one bit per allowed value of each field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cron {
    /// Minutes 0..=59.
    pub minutes: u64,
    /// Hours 0..=23.
    pub hours: u32,
    /// Days of the month 1..=31.
    pub days: u32,
    /// Months 1..=12.
    pub months: u16,
    /// Days of the week, Sunday = 0.
    pub weekdays: u8,
    /// The day-of-month field was `*`.
    pub any_day: bool,
    /// The day-of-week field was `*`.
    pub any_weekday: bool,
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const DAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

fn shorthand(text: &str) -> Option<&'static str> {
    Some(match text {
        "@hourly" => "0 * * * *",
        "@daily" | "@midnight" => "0 0 * * *",
        "@weekly" => "0 0 * * 0",
        "@monthly" => "0 0 1 * *",
        "@yearly" | "@annually" => "0 0 1 1 *",
        _ => return None,
    })
}

fn value(word: &str, names: &[&str], base: u32) -> Result<u32, String> {
    let lower = word.to_ascii_lowercase();
    if let Some(i) = names.iter().position(|n| *n == lower) {
        return Ok(base + u32::try_from(i).unwrap_or(0));
    }
    word.parse().map_err(|_| format!("{word} is not a number"))
}

/// The bits of one comma-separated part such as `1-5`, `*/15` or `mon-fri/2`.
fn part(text: &str, (lo, hi): (u32, u32), names: &[&str]) -> Result<u64, String> {
    let (range, step) = match text.split_once('/') {
        Some((r, s)) => (r, s.parse::<u32>().map_err(|_| format!("bad step {s}"))?),
        None => (text, 1),
    };
    if step == 0 {
        return Err("a step of 0".into());
    }
    let (from, to) = match range {
        "*" => (lo, hi),
        r => match r.split_once('-') {
            Some((a, b)) => (value(a, names, lo)?, value(b, names, lo)?),
            None if text.contains('/') => (value(r, names, lo)?, hi),
            None => {
                let v = value(r, names, lo)?;
                (v, v)
            }
        },
    };
    if from < lo || to > hi || from > to {
        return Err(format!("{text} is outside {lo}-{hi}"));
    }
    Ok((from..=to)
        .step_by(step as usize)
        .fold(0, |m, v| m | (1u64 << v)))
}

fn field(text: &str, bounds: (u32, u32), names: &[&str]) -> Result<u64, String> {
    text.split(',')
        .try_fold(0, |m, p| Ok(m | part(p, bounds, names)?))
}

/// Parse a schedule; the error says which field is wrong.
pub fn parse_cron(text: &str) -> Result<Cron, String> {
    let text = text.trim();
    let text = shorthand(text).unwrap_or(text);
    let f: Vec<&str> = text.split_whitespace().collect();
    let [m, h, dom, mon, dow] = f.as_slice() else {
        return Err(format!(
            "{text}: a schedule has five fields (minute hour day month weekday)"
        ));
    };
    let named = |what: &str, r: Result<u64, String>| r.map_err(|e| format!("{what}: {e}"));
    let weekdays = named("weekday", field(dow, (0, 7), &DAYS))?;
    let weekdays = (weekdays | (weekdays >> 7)) & 0x7f;
    Ok(Cron {
        minutes: named("minute", field(m, (0, 59), &[]))?,
        hours: u32::try_from(named("hour", field(h, (0, 23), &[]))?).unwrap_or(0),
        days: u32::try_from(named("day", field(dom, (1, 31), &[]))?).unwrap_or(0),
        months: u16::try_from(named("month", field(mon, (1, 12), &MONTHS))?).unwrap_or(0),
        weekdays: u8::try_from(weekdays).unwrap_or(0),
        any_day: *dom == "*",
        any_weekday: *dow == "*",
    })
}

fn bit(mask: u64, v: u32) -> bool {
    mask & (1u64 << v) != 0
}

/// Whether the schedule fires in the local minute `t`. Like cron, when both the day of the month
/// and the day of the week are restricted, either one matching is enough.
pub fn cron_matches(c: &Cron, t: LocalTime) -> bool {
    let (hour, minute) = (u32::from(t.minute / 60), u32::from(t.minute % 60));
    let sunday_first = (u32::from(weekday_index(weekday(t.date))) + 1) % 7;
    let dom = bit(u64::from(c.days), u32::from(t.date.day));
    let dow = bit(u64::from(c.weekdays), sunday_first);
    let day = match (c.any_day, c.any_weekday) {
        (true, true) => true,
        (false, true) => dom,
        (true, false) => dow,
        (false, false) => dom || dow,
    };
    bit(c.minutes, minute)
        && bit(u64::from(c.hours), hour)
        && bit(u64::from(c.months), u32::from(t.date.month))
        && day
}
