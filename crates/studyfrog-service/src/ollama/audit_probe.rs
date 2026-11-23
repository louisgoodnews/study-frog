//! The hardware audit's probes: read the system's files and run its read-only commands, then hand
//! the text to the parsers. A probe that fails leaves its values unknown.

use std::fs;
use std::path::{Path, PathBuf};

use super::audit_parse::{
    mac_x86_flags, parse_cpuinfo, parse_df_available, parse_meminfo, parse_number,
    parse_nvidia_smi, parse_vm_stat_free,
};
use super::audit_types::{Audit, CpuInfo, GpuInfo, Os};
use super::run::run_timeout;

/// Seconds a probe command may take.
const PROBE_SECS: u64 = 3;
/// PCI vendor id of AMD in sysfs.
const AMD_VENDOR: &str = "0x1002";

/// The operating system this build runs on.
pub fn current_os() -> Os {
    if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "windows") {
        Os::Windows
    } else {
        Os::Other
    }
}

/// What the audit needs from the caller: the system and the folder where models are stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inputs {
    /// The operating system.
    pub os: Os,
    /// The folder the models are stored in (its free space is reported), if known.
    pub models_dir: Option<PathBuf>,
}

/// Run the whole audit.
pub fn read_audit(inputs: &Inputs) -> Audit {
    let (cpu, ram_total, ram_free, gpus) = match inputs.os {
        Os::Linux => linux_hardware(),
        Os::MacOs => mac_hardware(),
        Os::Windows | Os::Other => (CpuInfo::default(), None, None, Vec::new()),
    };
    let disk_free = inputs
        .models_dir
        .as_deref()
        .and_then(nearest_existing)
        .and_then(|p| disk_free_at(&p));
    Audit {
        os: inputs.os,
        cpu,
        ram_total,
        ram_free,
        gpus,
        models_dir: inputs.models_dir.as_ref().map(|p| p.display().to_string()),
        disk_free,
    }
}

type Hardware = (CpuInfo, Option<u64>, Option<u64>, Vec<GpuInfo>);

fn linux_hardware() -> Hardware {
    let cpu = fs::read_to_string("/proc/cpuinfo")
        .map(|t| parse_cpuinfo(&t))
        .unwrap_or_default();
    let (total, free) = fs::read_to_string("/proc/meminfo")
        .map(|t| parse_meminfo(&t))
        .unwrap_or((None, None));
    let mut gpus = run_timeout(
        "nvidia-smi",
        &[
            "--query-gpu=name,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ],
        PROBE_SECS,
    )
    .map(|t| parse_nvidia_smi(&t))
    .unwrap_or_default();
    gpus.extend(sysfs_gpus());
    (cpu, total, free, gpus)
}

/// AMD cards (and Intel ones, without memory figures) listed under `/sys/class/drm`.
fn sysfs_gpus() -> Vec<GpuInfo> {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };
    let mut names: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.file_name().is_some_and(|n| is_card(&n.to_string_lossy())))
        .collect();
    names.sort();
    names.iter().filter_map(|p| amd_card(p)).collect()
}

fn is_card(name: &str) -> bool {
    name.strip_prefix("card")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

fn amd_card(card: &Path) -> Option<GpuInfo> {
    let device = card.join("device");
    let vendor = fs::read_to_string(device.join("vendor")).ok()?;
    if vendor.trim() != AMD_VENDOR {
        return None;
    }
    let total = read_u64(&device.join("mem_info_vram_total"));
    let used = read_u64(&device.join("mem_info_vram_used"));
    let card_name = card.file_name()?.to_string_lossy().into_owned();
    Some(GpuInfo {
        name: format!("AMD GPU ({card_name})"),
        vram_total: total,
        vram_free: total.zip(used).map(|(t, u)| t.saturating_sub(u)),
        shared: false,
    })
}

fn read_u64(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok().and_then(|t| parse_number(&t))
}

fn mac_hardware() -> Hardware {
    let text = |args: &[&str]| run_timeout("sysctl", args, PROBE_SECS).ok();
    let number = |key: &str| text(&["-n", key]).and_then(|t| parse_number(&t));
    let arm = number("hw.optional.arm64") == Some(1);
    let features = text(&["-n", "machdep.cpu.features"]).unwrap_or_default();
    let leaf7 = text(&["-n", "machdep.cpu.leaf7_features"]).unwrap_or_default();
    let mut cpu = mac_x86_flags(&features, &leaf7);
    cpu.model = text(&["-n", "machdep.cpu.brand_string"]).map(|t| t.trim().to_owned());
    cpu.logical_cores = number("hw.ncpu").and_then(|n| u32::try_from(n).ok());
    cpu.neon = arm.then_some(true);
    let free = run_timeout("vm_stat", &[], PROBE_SECS)
        .ok()
        .and_then(|t| parse_vm_stat_free(&t));
    let gpus = if arm {
        vec![GpuInfo {
            name: "Apple GPU".to_owned(),
            vram_total: None,
            vram_free: None,
            shared: true,
        }]
    } else {
        Vec::new()
    };
    (cpu, number("hw.memsize"), free, gpus)
}

/// The folder itself, or the nearest parent that exists (a models folder may not exist yet).
pub fn nearest_existing(path: &Path) -> Option<PathBuf> {
    path.ancestors().find(|p| p.exists()).map(Path::to_path_buf)
}

fn disk_free_at(path: &Path) -> Option<u64> {
    let arg = path.to_string_lossy().into_owned();
    run_timeout("df", &["-Pk", &arg], PROBE_SECS)
        .ok()
        .and_then(|t| parse_df_available(&t))
}
