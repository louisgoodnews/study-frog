//! Issue file model and loading. Validation rules live in `rules`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::files::find_files;

/// Front-matter exactly as specified in `docs/issues/README.md`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Front {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: String,
    pub epic: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    pub estimate: Option<u32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

/// A parsed issue file.
pub struct Issue {
    pub path: PathBuf,
    pub dir: String,
    pub front: Front,
    pub body: String,
}

/// Load every issue under `dir`. Returns the parsed issues and any load errors.
pub fn load_all(dir: &Path) -> (Vec<Issue>, Vec<String>) {
    let results: Vec<_> = find_files(dir, "md")
        .into_iter()
        .filter(|p| !is_readme(p))
        .map(|p| load_one(&p))
        .collect();
    let errors = results
        .iter()
        .filter_map(|r| r.as_ref().err().cloned())
        .collect();
    let loaded = results.into_iter().filter_map(Result::ok).collect();
    (loaded, errors)
}

fn is_readme(path: &Path) -> bool {
    path.file_name().is_some_and(|n| n == "README.md")
}

fn load_one(path: &Path) -> Result<Issue, String> {
    let label = path.display();
    let text = fs::read_to_string(path).map_err(|e| format!("{label}: {e}"))?;
    let (yaml, body) = split_front_matter(&text)
        .ok_or_else(|| format!("{label}: missing `---` front-matter block"))?;
    let front: Front =
        serde_saphyr::from_str(yaml).map_err(|e| format!("{label}: invalid front-matter: {e}"))?;
    Ok(Issue {
        path: path.to_path_buf(),
        dir: parent_dir_name(path),
        front,
        body: body.to_owned(),
    })
}

fn parent_dir_name(path: &Path) -> String {
    path.parent()
        .and_then(Path::file_name)
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// Split `---\n<yaml>\n---\n<body>` into `(yaml, body)`.
pub fn split_front_matter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest
        .find("\n---\n")
        .or_else(|| rest.find("\n---").filter(|i| i + 4 == rest.len()))?;
    let body_start = (end + 5).min(rest.len());
    Some((&rest[..end], &rest[body_start..]))
}
