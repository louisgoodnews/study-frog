//! The native "choose a folder" and "choose a file" dialogs, through the desktop portal (no GTK needed), and
//! the system program that shows a folder in the file manager.

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

/// The system's program that shows a folder: `xdg-open` on Linux and the BSDs, `open` on macOS,
/// `explorer` on Windows.
pub fn opener_program() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    }
}

/// Show `dir` in the system's file manager. The program is started and left to run; a thread
/// waits for it so it leaves no zombie process behind.
pub fn open_folder(dir: &std::path::Path) -> Result<(), String> {
    let program = opener_program();
    let mut child = std::process::Command::new(program)
        .arg(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("{program} could not be started: {e}"))?;
    std::thread::spawn(move || child.wait());
    Ok(())
}
