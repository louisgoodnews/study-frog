//! The familiarity filter of the run setup.

use studyfrog_core::plan::DifficultyRange;

use crate::setup_form::SetupForm;

pub(crate) fn number<T: std::str::FromStr>(text: &str, what: &str) -> Result<T, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{what} must be a number"))
}

pub(crate) fn limit(f: &SetupForm) -> Result<Option<u32>, String> {
    if f.limit.trim().is_empty() {
        return Ok(None);
    }
    match number::<u32>(&f.limit, "the limit")? {
        0 => Err("the limit must be at least 1".to_owned()),
        n => Ok(Some(n)),
    }
}

/// The bound the form asks for, as a fraction; `None` when the filter is off.
pub fn familiarity_bound(f: &SetupForm) -> Result<Option<f32>, String> {
    if !f.fam_on {
        return Ok(None);
    }
    let percent = number::<f32>(&f.fam_below, "the familiarity")?;
    if !(1.0..=100.0).contains(&percent) {
        return Err("familiarity runs from 1 to 100 percent".to_owned());
    }
    Ok(Some(percent / 100.0))
}

/// The difficulty range the form asks for; `None` when the filter is off.
pub fn difficulty(f: &SetupForm) -> Result<Option<DifficultyRange>, String> {
    if !f.difficulty_on {
        return Ok(None);
    }
    let (min, max): (f32, f32) = (
        number(&f.diff_min, "the lowest difficulty")?,
        number(&f.diff_max, "the highest difficulty")?,
    );
    if !(1.0..=10.0).contains(&min) || !(1.0..=10.0).contains(&max) || min > max {
        return Err("difficulty runs from 1 to 10, lowest first".to_owned());
    }
    Ok(Some(DifficultyRange { min, max }))
}
