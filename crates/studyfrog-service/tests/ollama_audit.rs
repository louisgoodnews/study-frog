#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The setup assistant's pure parts: the audit parsers on recorded samples, the fit rule, the
//! ranking and the guide.

use studyfrog_service::ollama::audit_parse::{
    mac_x86_flags, parse_cpuinfo, parse_df_available, parse_meminfo, parse_number,
    parse_nvidia_smi, parse_vm_stat_free,
};
use studyfrog_service::ollama::client::{is_loopback, parse_tags, parse_version};

const CPUINFO: &str = "processor\t: 0\nvendor_id\t: GenuineIntel\nmodel name\t: Intel(R) Core(TM) i7-8650U CPU @ 1.90GHz\nflags\t\t: fpu vme avx avx2 fma f16c sse4_2\n\nprocessor\t: 1\nmodel name\t: Intel(R) Core(TM) i7-8650U CPU @ 1.90GHz\nflags\t\t: fpu vme avx avx2 fma f16c sse4_2\n";

const MEMINFO: &str = "MemTotal:       16301068 kB\nMemFree:         1024000 kB\nMemAvailable:    9876543 kB\nBuffers:          100 kB\n";

const DF: &str = "Filesystem     1024-blocks      Used Available Capacity Mounted on\n/dev/vda        264000000 70000000  190000000      27% /\n";

const VM_STAT: &str = "Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free:                              12345.\nPages active:                           999.\n";

#[test]
fn cpuinfo_gives_the_model_cores_and_the_inference_instruction_sets() {
    let cpu = parse_cpuinfo(CPUINFO);
    assert_eq!(
        cpu.model.as_deref(),
        Some("Intel(R) Core(TM) i7-8650U CPU @ 1.90GHz")
    );
    assert_eq!(cpu.logical_cores, Some(2));
    assert_eq!(
        (cpu.avx, cpu.avx2, cpu.fma, cpu.f16c),
        (Some(true), Some(true), Some(true), Some(true))
    );
    assert_eq!(cpu.neon, Some(false));
}

#[test]
fn cpuinfo_without_flags_leaves_the_sets_unknown() {
    let cpu = parse_cpuinfo("processor\t: 0\nmodel name\t: Mystery\n");
    assert_eq!(cpu.avx2, None);
    assert_eq!(cpu.logical_cores, Some(1));
}

#[test]
fn arm_cpuinfo_reports_neon_from_the_features_line() {
    let cpu =
        parse_cpuinfo("processor\t: 0\nFeatures\t: fp asimd evtstrm aes\nHardware\t: BCM2835\n");
    assert_eq!(cpu.neon, Some(true));
    assert_eq!(cpu.avx2, Some(false), "an Arm flags line lists no AVX2");
    assert_eq!(cpu.model.as_deref(), Some("BCM2835"));
}

#[test]
fn meminfo_gives_total_and_available_in_bytes() {
    let (total, free) = parse_meminfo(MEMINFO);
    assert_eq!(total, Some(16_301_068 * 1024));
    assert_eq!(free, Some(9_876_543 * 1024));
    assert_eq!(parse_meminfo("nothing here"), (None, None));
}

#[test]
fn nvidia_smi_lines_give_name_and_video_memory_in_bytes() {
    let gpus = parse_nvidia_smi("NVIDIA GeForce RTX 3060, 12288, 9000\n");
    assert_eq!(gpus.len(), 1);
    assert_eq!(gpus[0].name, "NVIDIA GeForce RTX 3060");
    assert_eq!(gpus[0].vram_total, Some(12288 * 1024 * 1024));
    assert_eq!(gpus[0].vram_free, Some(9000 * 1024 * 1024));
    assert!(!gpus[0].shared);
}

#[test]
fn nvidia_smi_values_it_cannot_read_stay_unknown() {
    let gpus = parse_nvidia_smi("Tesla T4, [N/A], [Not Supported]\nbroken line\n");
    assert_eq!(gpus.len(), 1);
    assert_eq!(gpus[0].vram_total, None);
    assert_eq!(gpus[0].vram_free, None);
}

#[test]
fn df_gives_the_available_bytes_of_the_second_row() {
    assert_eq!(parse_df_available(DF), Some(190_000_000 * 1024));
    assert_eq!(parse_df_available("Filesystem\n"), None);
}

#[test]
fn vm_stat_gives_free_pages_times_the_page_size() {
    assert_eq!(parse_vm_stat_free(VM_STAT), Some(12345 * 16384));
    assert_eq!(parse_vm_stat_free("no header"), None);
}

#[test]
fn mac_flags_come_from_the_feature_lists() {
    let cpu = mac_x86_flags("FPU VME AVX1.0 FMA F16C SSE4.2", "BMI1 AVX2 BMI2");
    assert_eq!(
        (cpu.avx, cpu.avx2, cpu.fma, cpu.f16c),
        (Some(true), Some(true), Some(true), Some(true))
    );
    let none = mac_x86_flags("", "");
    assert_eq!(none.avx2, None);
}

#[test]
fn a_plain_number_parses_and_junk_does_not() {
    assert_eq!(parse_number("17179869184\n"), Some(17_179_869_184));
    assert_eq!(parse_number("n/a"), None);
}

#[test]
fn version_and_tags_parse_the_service_answers() {
    assert_eq!(
        parse_version(r#"{"version":"0.5.4"}"#),
        Some("0.5.4".to_owned())
    );
    assert_eq!(parse_version("<html>"), None);
    let tags = parse_tags(
        r#"{"models":[{"name":"qwen2.5:1.5b","size":986000000,"details":{"parameter_size":"1.5B","quantization_level":"Q4_K_M"}}]}"#,
    )
    .expect("tags");
    assert_eq!(tags[0].name, "qwen2.5:1.5b");
    assert_eq!(tags[0].size_bytes, Some(986_000_000));
    assert_eq!(tags[0].quantization.as_deref(), Some("Q4_K_M"));
    assert!(parse_tags(r#"{"nothing":1}"#).is_err());
}

#[test]
fn loopback_addresses_only() {
    assert!(is_loopback("http://127.0.0.1:11434"));
    assert!(is_loopback("http://localhost:11434/"));
    assert!(is_loopback("http://[::1]:11434"));
    assert!(!is_loopback("http://10.0.0.5:11434"));
    assert!(!is_loopback("http://localhost.evil.example:11434"));
    assert!(!is_loopback("http://127.0.0.1.evil.example:11434"));
}
