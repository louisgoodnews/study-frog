//! Multi-tab ScriptStudio workbench and script manager for FrogScript and Python scripts.

use std::path::{Path, PathBuf};

/// A script open in an editor tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptTab {
    /// Tab display title (e.g. `cram_session.frog`).
    pub name: String,
    /// Absolute or vault-relative path on disk if saved.
    pub path: Option<PathBuf>,
    /// Script source text.
    pub content: String,
    /// Unsaved modifications.
    pub dirty: bool,
}

/// Output log stream filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleFilter {
    /// All entries.
    #[default]
    All,
    /// Stdout messages and user commands.
    Stdout,
    /// Diagnostic stderr traces and errors.
    Stderr,
    /// Lifecycle and application events.
    Events,
}

/// State of the ScriptStudio environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioState {
    /// Open script tabs.
    pub tabs: Vec<ScriptTab>,
    /// Currently active tab index.
    pub active_tab: usize,
    /// Active log filter.
    pub filter: ConsoleFilter,
    /// Discovered script files from vault and app directories.
    pub available_scripts: Vec<PathBuf>,
    /// Whether ScriptStudio panel is expanded in console.
    pub expanded: bool,
}

const DEFAULT_RECIPE: &str = "# FrogScript Study Automation
cards = from stack * | where type \"card\"
log_info(\"Inspecting items\")
cards | where familiarity < 0.7 | set priority \"high\"
";

/// Create a new tab with given name, content, and optional path.
pub fn tab_new(
    name: impl Into<String>,
    content: impl Into<String>,
    path: Option<PathBuf>,
) -> ScriptTab {
    ScriptTab {
        name: name.into(),
        content: content.into(),
        path,
        dirty: false,
    }
}

/// Create a default StudioState with one initial draft recipe tab.
pub fn studio_new() -> StudioState {
    StudioState {
        tabs: vec![tab_new("quickstart.frog", DEFAULT_RECIPE, None)],
        active_tab: 0,
        filter: ConsoleFilter::All,
        available_scripts: Vec::new(),
        expanded: false,
    }
}

impl Default for StudioState {
    fn default() -> Self {
        studio_new()
    }
}

/// Currently active tab reference.
pub fn active_tab(s: &StudioState) -> Option<&ScriptTab> {
    s.tabs.get(s.active_tab)
}

/// Currently active tab mutable reference.
pub fn active_tab_mut(s: &mut StudioState) -> Option<&mut ScriptTab> {
    s.tabs.get_mut(s.active_tab)
}

/// Switch active tab index.
pub fn select_tab(s: &mut StudioState, idx: usize) {
    if idx < s.tabs.len() {
        s.active_tab = idx;
    }
}

/// Close tab by index, maintaining a valid active tab.
pub fn close_tab(s: &mut StudioState, idx: usize) {
    if idx < s.tabs.len() {
        s.tabs.remove(idx);
        if s.tabs.is_empty() {
            s.tabs.push(tab_new("untitled.frog", "", None));
            s.active_tab = 0;
        } else if s.active_tab >= s.tabs.len() {
            s.active_tab = s.tabs.len().saturating_sub(1);
        }
    }
}

/// Open a new empty tab.
pub fn new_tab(s: &mut StudioState, name: Option<&str>) {
    let title = name.unwrap_or("untitled.frog");
    s.tabs.push(tab_new(title, "", None));
    s.active_tab = s.tabs.len().saturating_sub(1);
}

/// Update source text of currently active tab.
pub fn set_active_code(s: &mut StudioState, code: String) {
    if let Some(tab) = active_tab_mut(s)
        && tab.content != code
    {
        tab.content = code;
        tab.dirty = true;
    }
}

/// Open a script from path or switch to it if already open.
pub fn open_script(s: &mut StudioState, path: &Path, content: &str) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("script.frog")
        .to_string();

    for (i, tab) in s.tabs.iter().enumerate() {
        if tab.path.as_deref() == Some(path) {
            s.active_tab = i;
            return;
        }
    }

    s.tabs
        .push(tab_new(name, content, Some(path.to_path_buf())));
    s.active_tab = s.tabs.len().saturating_sub(1);
}

/// Save active script to its existing path or vault's `.studyfrog/scripts/`.
pub fn save_active_script(
    s: &mut StudioState,
    vault_root: Option<&Path>,
) -> Result<PathBuf, String> {
    let tab = active_tab_mut(s).ok_or_else(|| "no open tab to save".to_string())?;

    let target = match &tab.path {
        Some(p) => p.clone(),
        None => {
            let base = vault_root
                .map(|v| v.join(".studyfrog").join("scripts"))
                .unwrap_or_else(|| PathBuf::from("scripts"));
            std::fs::create_dir_all(&base)
                .map_err(|e| format!("cannot create scripts directory: {e}"))?;
            base.join(&tab.name)
        }
    };

    std::fs::write(&target, &tab.content)
        .map_err(|e| format!("cannot write file {}: {e}", target.display()))?;

    tab.path = Some(target.clone());
    tab.dirty = false;
    Ok(target)
}

/// Resolve the application-level scripts folder from environment or user home.
pub fn resolve_app_scripts() -> Option<PathBuf> {
    let own = std::env::var(studyfrog_config::paths::HOME_ENV).ok();
    let data = std::env::var("XDG_DATA_HOME").ok();
    let home = std::env::var("HOME").ok();
    let root = studyfrog_config::paths::app_home(own.as_deref(), data.as_deref(), home.as_deref())?;
    Some(studyfrog_config::paths::app_paths(&root).scripts)
}

/// Discover all scripts in vault `.studyfrog/scripts/` and application scripts folder.
pub fn discover_scripts(vault_root: Option<&Path>, app_scripts: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let mut scan_dir = |dir: &Path| {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext == "frog" || ext == "py" || ext == "sh" {
                        out.push(p);
                    }
                }
            }
        }
    };

    if let Some(vault) = vault_root {
        scan_dir(&vault.join(".studyfrog").join("scripts"));
    }
    scan_dir(app_scripts);

    out.sort();
    out.dedup();
    out
}
