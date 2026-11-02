//! The vault's saved library filters: `filters.toml`.

use std::fs;

use studyfrog_config::error::describe_config_error;
use studyfrog_config::filters::{
    FILTERS_FILE, SavedFilter, parse_filters, render_filters, with_filter, without_filter,
};

use crate::create::write_new;
use crate::session::{ServiceError, VaultSession};

/// The saved filters, or none with the reason when the file cannot be used.
pub fn load_filters(session: &VaultSession) -> (Vec<SavedFilter>, Option<String>) {
    match fs::read_to_string(session.root.join(FILTERS_FILE)) {
        Ok(text) => match parse_filters(&text) {
            Ok(list) => (list, None),
            Err(e) => (
                Vec::new(),
                Some(format!(
                    "{FILTERS_FILE} is not used: {}",
                    describe_config_error(&e)
                )),
            ),
        },
        Err(_) => (Vec::new(), None),
    }
}

/// Save a filter under its name (replacing one of that name) and return the list now stored.
pub fn save_filter(
    session: &VaultSession,
    filter: SavedFilter,
) -> Result<Vec<SavedFilter>, ServiceError> {
    let (list, unusable) = load_filters(session);
    if let Some(why) = unusable {
        return Err(ServiceError::Invalid(why));
    }
    let next = with_filter(&list, filter).map_err(ServiceError::Invalid)?;
    write_new(&session.root.join(FILTERS_FILE), &render_filters(&next))?;
    Ok(next)
}

/// Remove the filter of that name and return the list now stored.
pub fn delete_filter(session: &VaultSession, name: &str) -> Result<Vec<SavedFilter>, ServiceError> {
    let (list, unusable) = load_filters(session);
    if let Some(why) = unusable {
        return Err(ServiceError::Invalid(why));
    }
    let next = without_filter(&list, name);
    write_new(&session.root.join(FILTERS_FILE), &render_filters(&next))?;
    Ok(next)
}
