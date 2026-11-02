//! The familiarity filter of the run setup.

use crate::setup_form::{SetupForm, number};

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
