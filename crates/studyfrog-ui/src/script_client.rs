//! The Python client (`python/studyfrog/`) carried inside the binary, so scripts run by StudyFrog can
//! `import studyfrog` without installing anything: it is written to `<app home>/scripts/lib/`.

use std::fs;
use std::io;
use std::path::Path;

/// The package's files: name and text.
pub const CLIENT_FILES: &[(&str, &str)] = &[
    (
        "__init__.py",
        include_str!("../../../python/studyfrog/__init__.py"),
    ),
    (
        "client.py",
        include_str!("../../../python/studyfrog/client.py"),
    ),
    (
        "script.py",
        include_str!("../../../python/studyfrog/script.py"),
    ),
    (
        "transport.py",
        include_str!("../../../python/studyfrog/transport.py"),
    ),
    (
        "types.py",
        include_str!("../../../python/studyfrog/types.py"),
    ),
];

/// The folder put on `PYTHONPATH` for scripts: `lib/` inside the scripts folder.
pub fn lib_dir(scripts: &Path) -> std::path::PathBuf {
    scripts.join("lib")
}

/// Write the package to `<dir>/studyfrog/`, leaving files that are already the same; how many files
/// were written.
pub fn install_client(dir: &Path) -> io::Result<usize> {
    let pkg = dir.join("studyfrog");
    fs::create_dir_all(&pkg)?;
    let mut written = 0;
    for (name, text) in CLIENT_FILES {
        let path = pkg.join(name);
        if fs::read_to_string(&path).ok().as_deref() == Some(*text) {
            continue;
        }
        fs::write(&path, text)?;
        written += 1;
    }
    Ok(written)
}
