//! The user's own help pages: Markdown files in the vault's `wiki/` folder. The vault scanner leaves the
//! folder out of the index, so these pages are not items; backups take every file and so take them too.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_store::vault::WIKI_DIR;

use crate::help::UserPage;
use crate::session::ServiceError;

/// Longest file name of a page, in characters.
pub const MAX_NAME: usize = 80;

fn io(path: &Path, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

fn dir(root: &Path) -> PathBuf {
    root.join(WIKI_DIR)
}

/// Why `name` cannot be the name of a page, or `None` when it can.
pub fn name_problem(name: &str) -> Option<&'static str> {
    let n = name.trim();
    if n.is_empty() {
        Some("Give the page a name.")
    } else if n.chars().count() > MAX_NAME {
        Some("The name is longer than 80 characters.")
    } else if n.starts_with('.') {
        Some("The name cannot start with a dot.")
    } else if n
        .chars()
        .any(|c| matches!(c, '/' | '\\' | ':' | '\0') || c.is_control())
    {
        Some("The name cannot contain / \\ or :.")
    } else {
        None
    }
}

fn path_of(root: &Path, name: &str) -> Result<PathBuf, ServiceError> {
    match name_problem(name) {
        Some(why) => Err(ServiceError::Invalid(why.to_owned())),
        None => Ok(dir(root).join(format!("{}.md", name.trim()))),
    }
}

fn read_page(path: &Path) -> Option<Result<UserPage, ServiceError>> {
    let is_md = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("md"));
    let file = path.file_stem()?.to_string_lossy().into_owned();
    if !is_md || file.starts_with('.') || !path.is_file() {
        return None;
    }
    Some(
        fs::read_to_string(path)
            .map(|text| UserPage { file, text })
            .map_err(|e| io(path, &e)),
    )
}

/// The pages in `wiki/`, sorted by file name; none when the folder does not exist.
pub fn list_user_pages(root: &Path) -> Result<Vec<UserPage>, ServiceError> {
    let folder = dir(root);
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&folder).map_err(|e| io(&folder, &e))?;
    let mut pages = entries
        .filter_map(Result::ok)
        .filter_map(|e| read_page(&e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    pages.sort_by_key(|p| p.file.to_lowercase());
    Ok(pages)
}

fn write(path: &Path, text: &str) -> Result<(), ServiceError> {
    let parent = path.parent().unwrap_or(path);
    fs::create_dir_all(parent).map_err(|e| io(parent, &e))?;
    let temp = path.with_extension("md.tmp");
    fs::write(&temp, text).map_err(|e| io(&temp, &e))?;
    fs::rename(&temp, path).map_err(|e| io(path, &e))
}

/// Write the page `name`. `old` is the name it had (`None` for a new page): a different name renames the
/// file. Refuses to overwrite another page. Returns the pages after the change.
pub fn save_user_page(
    root: &Path,
    old: Option<&str>,
    name: &str,
    text: &str,
) -> Result<Vec<UserPage>, ServiceError> {
    let target = path_of(root, name)?;
    let same = old.is_some_and(|o| o.trim() == name.trim());
    let other = |p: &UserPage| old.is_none_or(|o| p.file != o.trim());
    let clash = list_user_pages(root)?
        .iter()
        .any(|p| other(p) && p.file.eq_ignore_ascii_case(name.trim()));
    if !same && (clash || target.exists()) {
        let why = format!("A page named \"{}\" exists already.", name.trim());
        return Err(ServiceError::Invalid(why));
    }
    write(&target, text)?;
    if let Some(previous) = old.filter(|_| !same) {
        let from = path_of(root, previous)?;
        fs::remove_file(&from).map_err(|e| io(&from, &e))?;
    }
    list_user_pages(root)
}

/// Delete the page `name`. Returns the pages after the change.
pub fn delete_user_page(root: &Path, name: &str) -> Result<Vec<UserPage>, ServiceError> {
    let path = path_of(root, name)?;
    fs::remove_file(&path).map_err(|e| io(&path, &e))?;
    list_user_pages(root)
}

/// A name for a new page that no page has yet: `base`, then `base 2`, `base 3` and so on.
pub fn free_name(base: &str, pages: &[UserPage]) -> String {
    let taken = |n: &str| pages.iter().any(|p| p.file.eq_ignore_ascii_case(n));
    (1..)
        .map(|i| match i {
            1 => base.to_owned(),
            _ => format!("{base} {i}"),
        })
        .find(|n| !taken(n))
        .unwrap_or_else(|| base.to_owned())
}
