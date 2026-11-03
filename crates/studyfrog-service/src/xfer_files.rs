//! Copying the pictures and PDFs of an import bundle into the vault.

use std::fs;
use std::path::Path;

use crate::session::VaultSession;
use crate::xfer_model::{Bundle, safe_folder};

pub(crate) fn copy_files(session: &VaultSession, b: &Bundle) {
    for (path, bytes) in &b.files {
        let ok = safe_folder(path) && Path::new(path).extension().is_some();
        let target = session.root.join(path);
        if ok && !target.exists() {
            if let Some(dir) = target.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = fs::write(target, bytes);
        }
    }
}
