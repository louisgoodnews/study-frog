//! The native "choose a folder" dialog, through the desktop portal (no GTK needed).

use std::path::PathBuf;

/// Ask the user for a file with one of these extensions (no dot). `None` when they cancel or the
/// portal is not available.
pub fn pick_file(extensions: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose a file to import")
        .add_filter("Importable files", extensions)
        .pick_file()
}

/// Ask the user for a folder. `None` when they cancel or the portal is not available.
pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose your vault folder")
        .pick_folder()
}
