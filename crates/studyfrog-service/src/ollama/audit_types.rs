//! What the hardware audit found. A value that could not be read is `None` and is shown as unknown,
//! never guessed.

/// The operating system family of this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Os {
    /// Linux (also the other Unix systems that expose `/proc`).
    Linux,
    /// macOS.
    MacOs,
    /// Windows (the audit reads no values there yet).
    Windows,
    /// Anything else.
    #[default]
    Other,
}

/// The processor. Instruction sets are `None` when they could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CpuInfo {
    /// The model name as the system reports it.
    pub model: Option<String>,
    /// Logical cores (threads).
    pub logical_cores: Option<u32>,
    /// AVX (x86).
    pub avx: Option<bool>,
    /// AVX2 (x86), the set most inference runtimes use.
    pub avx2: Option<bool>,
    /// FMA (x86).
    pub fma: Option<bool>,
    /// F16C (x86), half-precision conversion.
    pub f16c: Option<bool>,
    /// NEON / Advanced SIMD (Arm).
    pub neon: Option<bool>,
}

/// A graphics card. `vram_*` are bytes; `None` when the card does not report them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInfo {
    /// The name the card reports.
    pub name: String,
    /// Total video memory in bytes.
    pub vram_total: Option<u64>,
    /// Free video memory in bytes.
    pub vram_free: Option<u64>,
    /// The card uses the computer's own memory (Apple silicon): its memory is the computer's free memory.
    pub shared: bool,
}

/// The whole audit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Audit {
    /// The operating system.
    pub os: Os,
    /// The processor.
    pub cpu: CpuInfo,
    /// Total memory in bytes.
    pub ram_total: Option<u64>,
    /// Memory available for new work in bytes.
    pub ram_free: Option<u64>,
    /// The graphics cards that could be read (empty when none could be read).
    pub gpus: Vec<GpuInfo>,
    /// The folder the models are stored in, as it was checked.
    pub models_dir: Option<String>,
    /// Free bytes on the disk that holds the models folder.
    pub disk_free: Option<u64>,
}
