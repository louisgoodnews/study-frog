//! Parsers for the text the system reports (`/proc`, `sysctl`, `vm_stat`, `nvidia-smi`, `df`). Pure:
//! each takes the text and returns what it could read; anything missing is `None`.

use super::audit_types::{CpuInfo, GpuInfo};

const MIB: u64 = 1024 * 1024;

/// The processor from `/proc/cpuinfo`. The model and the instruction sets come from the first
/// processor's lines; the core count is the number of processors listed.
pub fn parse_cpuinfo(text: &str) -> CpuInfo {
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.trim_start().strip_prefix(':'))
            .map(|v| v.trim().to_owned())
    };
    let logical = text.lines().filter(|l| l.starts_with("processor")).count();
    let flags = field("flags")
        .or_else(|| field("Features"))
        .unwrap_or_default();
    let words: Vec<&str> = flags.split_whitespace().collect();
    let has = |w: &str| (!flags.is_empty()).then(|| words.contains(&w));
    CpuInfo {
        model: field("model name").or_else(|| field("Hardware")),
        logical_cores: u32::try_from(logical).ok().filter(|n| *n > 0),
        avx: has("avx"),
        avx2: has("avx2"),
        fma: has("fma"),
        f16c: has("f16c"),
        neon: (!flags.is_empty()).then(|| words.contains(&"asimd") || words.contains(&"neon")),
    }
}

/// `MemTotal` and `MemAvailable` from `/proc/meminfo`, in bytes.
pub fn parse_meminfo(text: &str) -> (Option<u64>, Option<u64>) {
    let kib = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix(':'))
            .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
            .map(|k| k * 1024)
    };
    (kib("MemTotal"), kib("MemAvailable"))
}

/// Lines of `nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv,noheader,nounits`:
/// the name, then the total and the free video memory in MiB.
pub fn parse_nvidia_smi(text: &str) -> Vec<GpuInfo> {
    text.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
            let [name, total, free] = parts.as_slice() else {
                return None;
            };
            if name.is_empty() {
                return None;
            }
            Some(GpuInfo {
                name: (*name).to_owned(),
                vram_total: mib(total),
                vram_free: mib(free),
                shared: false,
            })
        })
        .collect()
}

fn mib(text: &str) -> Option<u64> {
    text.parse::<u64>().ok().map(|m| m * MIB)
}

/// The available bytes of the filesystem from `df -Pk` (POSIX output, 1024-byte blocks).
pub fn parse_df_available(text: &str) -> Option<u64> {
    let row = text.lines().nth(1)?;
    let kib: u64 = row.split_whitespace().nth(3)?.parse().ok()?;
    Some(kib * 1024)
}

/// A single number, such as the output of `sysctl -n hw.memsize` or a sysfs file.
pub fn parse_number(text: &str) -> Option<u64> {
    text.trim().parse().ok()
}

/// Free bytes from `vm_stat`: the free pages times the page size in its header.
pub fn parse_vm_stat_free(text: &str) -> Option<u64> {
    let page = text
        .split("page size of ")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    let free = text
        .lines()
        .find_map(|l| l.strip_prefix("Pages free:"))?
        .trim()
        .trim_end_matches('.')
        .parse::<u64>()
        .ok()?;
    Some(free * page)
}

/// The instruction sets of an Intel or AMD Mac from `machdep.cpu.features` (`features`) and
/// `machdep.cpu.leaf7_features` (`leaf7`). `None` for each set when the text is empty.
pub fn mac_x86_flags(features: &str, leaf7: &str) -> CpuInfo {
    let known = !features.trim().is_empty();
    let has = |text: &str, w: &str| known.then(|| text.split_whitespace().any(|x| x == w));
    CpuInfo {
        avx: has(features, "AVX1.0"),
        avx2: (!leaf7.trim().is_empty()).then(|| leaf7.split_whitespace().any(|x| x == "AVX2")),
        fma: has(features, "FMA"),
        f16c: has(features, "F16C"),
        ..CpuInfo::default()
    }
}
