//! The native "choose a folder" dialog, through the desktop portal (no GTK needed).

use std::path::PathBuf;

/// Ask the user for a folder. `None` when they cancel or the portal is not available.
pub fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Choose your vault folder")
        .pick_folder()
}
