//! Reading and writing zip files for export and import, with limits so a hostile file cannot exhaust memory.

use std::io::{Cursor, Read, Write};

use zip::write::SimpleFileOptions;

/// Most files read from one zip.
pub const MAX_FILES: usize = 5_000;
/// Most bytes read from one zip entry.
pub const MAX_ENTRY: u64 = 64 * 1024 * 1024;
/// Most bytes read from one zip in all.
pub const MAX_TOTAL: u64 = 256 * 1024 * 1024;

/// A file of a zip: its relative path and its bytes.
pub type Entry = (String, Vec<u8>);

/// A zip of `(path, bytes)` entries.
pub fn write_zip(entries: &[Entry]) -> Result<Vec<u8>, String> {
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        w.start_file(name.as_str(), SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        w.write_all(bytes).map_err(|e| e.to_string())?;
    }
    Ok(w.finish().map_err(|e| e.to_string())?.into_inner())
}

/// The files of a zip: safe relative paths only, directories left out, sizes limited.
pub fn read_zip(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a zip file: {e}"))?;
    if archive.len() > MAX_FILES {
        return Err(format!(
            "the zip holds {} files; at most {MAX_FILES} are read",
            archive.len()
        ));
    }
    let (mut out, mut total) = (Vec::new(), 0_u64);
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("a zip entry cannot be read: {e}"))?;
        let Some(path) = entry.enclosed_name().filter(|_| entry.is_file()) else {
            continue;
        };
        let name = path.to_string_lossy().replace('\\', "/");
        let mut data = Vec::new();
        entry
            .take(MAX_ENTRY + 1)
            .read_to_end(&mut data)
            .map_err(|e| format!("{name}: {e}"))?;
        total += data.len() as u64;
        if data.len() as u64 > MAX_ENTRY || total > MAX_TOTAL {
            return Err("the zip is too large to read".to_owned());
        }
        out.push((name, data));
    }
    Ok(out)
}
