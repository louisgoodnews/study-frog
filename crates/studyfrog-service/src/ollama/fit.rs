//! The fit rule and the ranking of the suggestions. Pure: the same audit and installed models always
//! give the same list. A model whose size or memory need is unknown never counts as fitting.

use std::cmp::Reverse;

use super::audit_types::{Audit, GpuInfo};
use super::catalog::CatalogModel;
use super::client::Installed;

/// Share of free video memory a model may use, in per mille (the rest is left for the system).
pub const VIDEO_HEADROOM_PER_MILLE: u64 = 900;
/// Share of free memory a model may use when it runs on the processor, in per mille.
pub const MEMORY_HEADROOM_PER_MILLE: u64 = 800;

/// Where a model runs on this computer, or why it does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Fits in free video memory (fastest).
    Video,
    /// Fits in free memory and runs on the processor.
    Memory,
    /// Needs more than is free.
    TooLarge,
    /// Its need, or this computer's free memory, is not known.
    Unknown,
}

/// Whether a fit means the model runs here.
pub fn fits(fit: Fit) -> bool {
    matches!(fit, Fit::Video | Fit::Memory)
}

/// The free video memory of the first card that reports its own (not shared) memory.
pub fn vram_free_of(gpus: &[GpuInfo]) -> Option<u64> {
    gpus.iter().filter(|g| !g.shared).find_map(|g| g.vram_free)
}

/// Whether a model needing `need` bytes fits: in video memory when that is known and enough, else in
/// free memory. An unknown need is `Unknown`; so is an unknown free memory when video memory does not
/// hold the model either.
pub fn fit_of(need: Option<u64>, vram_free: Option<u64>, ram_free: Option<u64>) -> Fit {
    let Some(need) = need else {
        return Fit::Unknown;
    };
    if vram_free.is_some_and(|v| need <= share(v, VIDEO_HEADROOM_PER_MILLE)) {
        return Fit::Video;
    }
    match ram_free {
        Some(r) if need <= share(r, MEMORY_HEADROOM_PER_MILLE) => Fit::Memory,
        Some(_) => Fit::TooLarge,
        None => Fit::Unknown,
    }
}

fn share(bytes: u64, per_mille: u64) -> u64 {
    bytes / 1000 * per_mille
}

/// Higher is better.
fn tier(fit: Fit) -> u8 {
    match fit {
        Fit::Video => 3,
        Fit::Memory => 2,
        Fit::Unknown => 1,
        Fit::TooLarge => 0,
    }
}

/// One line of the suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// 1 for the best.
    pub rank: usize,
    /// The tag to pull.
    pub name: String,
    /// Parameter count as written.
    pub parameters: String,
    /// Quantization.
    pub quantization: String,
    /// Download size in bytes; `None` when unknown.
    pub download_bytes: Option<u64>,
    /// Memory needed in bytes; `None` when unknown.
    pub memory_bytes: Option<u64>,
    /// Where it fits.
    pub fit: Fit,
    /// Whether Ollama already has it.
    pub installed: bool,
    /// A short reason for the rank, in plain words.
    pub reason: String,
}

/// Rank the catalog for this audit. Fitting models first (video memory before processor), then the
/// ones already installed, then the larger models (more capable), then by name.
pub fn rank(catalog: &[CatalogModel], audit: &Audit, installed: &[Installed]) -> Vec<Suggestion> {
    let vram = vram_free_of(&audit.gpus);
    let mut keyed: Vec<(u32, Suggestion)> = catalog
        .iter()
        .map(|m| {
            let fit = fit_of(m.memory_bytes, vram, audit.ram_free);
            let have = installed.iter().any(|i| i.name == m.name);
            let s = Suggestion {
                rank: 0,
                name: m.name.clone(),
                parameters: m.parameters.clone(),
                quantization: m.quantization.clone(),
                download_bytes: m.download_bytes,
                memory_bytes: m.memory_bytes,
                fit,
                installed: have,
                reason: reason_of(fit, have),
            };
            (m.parameters_millions, s)
        })
        .collect();
    keyed.sort_by_key(|(params, s)| {
        (
            Reverse(tier(s.fit)),
            Reverse(s.installed),
            Reverse(*params),
            s.name.clone(),
        )
    });
    keyed
        .into_iter()
        .enumerate()
        .map(|(n, (_, mut s))| {
            s.rank = n + 1;
            s
        })
        .collect()
}

fn reason_of(fit: Fit, installed: bool) -> String {
    let have = if installed {
        " It is already installed."
    } else {
        ""
    };
    let text = match fit {
        Fit::Video => "Fits in video memory, the fastest place to run it.",
        Fit::Memory => "Fits in free memory; it runs on the processor, so answers are slower.",
        Fit::TooLarge => "Needs more memory than is free on this computer.",
        Fit::Unknown => "Its size or this computer's free memory is unknown, so fit is not known.",
    };
    format!("{text}{have}")
}
