//! Pictures for notes, cards and questions: copied into the vault's `assets/` folder, named after the
//! file and its content, so adding the same picture twice reuses the copy.

use std::fs;
use std::path::Path;

use studyfrog_store::hash::hash_bytes;

use crate::create::slug;
use crate::session::{ServiceError, VaultSession};

/// Folder of the pictures, relative to the vault root.
pub const ASSETS_DIR: &str = "assets";
/// Largest picture accepted, in bytes.
pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
const EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "gif"];

fn invalid(why: impl Into<String>) -> ServiceError {
    ServiceError::Invalid(why.into())
}

/// Copy the picture at `source` into the vault; returns its vault-relative path (`assets/name.png`).
pub fn add_image(session: &VaultSession, source: &Path) -> Result<String, ServiceError> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|e| EXTENSIONS.contains(&e.as_str()))
        .ok_or_else(|| invalid("only png, jpg and gif pictures can be added"))?;
    let io = |e: std::io::Error| ServiceError::Io(format!("{}: {e}", source.display()));
    let len = fs::metadata(source).map_err(io)?.len();
    if len > MAX_IMAGE_BYTES {
        return Err(invalid("that picture is larger than 20 MB"));
    }
    let bytes = fs::read(source).map_err(io)?;
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("picture");
    store_image(session, stem, &ext, &bytes)
}

/// Store picture bytes in the vault's `assets/` folder under a name made from `stem` and the content;
/// the same bytes reuse the same file. Returns the vault-relative path.
pub fn store_image(
    session: &VaultSession,
    stem: &str,
    ext: &str,
    bytes: &[u8],
) -> Result<String, ServiceError> {
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err(invalid("that picture is larger than 20 MB"));
    }
    let name = format!("{}-{:08x}.{ext}", slug(stem), hash_bytes(bytes) as u32);
    let dir = session.root.join(ASSETS_DIR);
    fs::create_dir_all(&dir).map_err(|e| ServiceError::Io(format!("{}: {e}", dir.display())))?;
    let target = dir.join(&name);
    if !target.exists() {
        fs::write(&target, bytes)
            .map_err(|e| ServiceError::Io(format!("{}: {e}", target.display())))?;
    }
    Ok(format!("{ASSETS_DIR}/{name}"))
}
