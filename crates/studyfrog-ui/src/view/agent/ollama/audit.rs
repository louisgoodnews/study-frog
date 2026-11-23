//! The hardware audit and the setup guide of the local-model panel (STORY-0206). Values that could
//! not be read say "unknown".

use eframe::egui::{self, RichText};
use studyfrog_service::ollama::audit_types::{Audit, CpuInfo, GpuInfo};
use studyfrog_service::ollama::fit::{Fit, Suggestion};
use studyfrog_service::ollama::guide::Step;

use crate::theme::palette_of;
use crate::view::kit;

/// A byte count in words (decimal units, as Ollama reports sizes), or "unknown".
pub fn bytes_text(bytes: Option<u64>) -> String {
    match bytes {
        None => "unknown".to_owned(),
        Some(b) if b >= 1_000_000_000 => format!("{:.1} GB", b as f64 / 1e9),
        Some(b) if b >= 1_000_000 => format!("{:.0} MB", b as f64 / 1e6),
        Some(b) => format!("{b} B"),
    }
}

/// A yes, no or unknown answer in words.
fn yes_no(v: Option<bool>) -> &'static str {
    match v {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

/// The processor's line: the model, the cores and the instruction sets that matter for inference.
pub fn cpu_text(cpu: &CpuInfo) -> String {
    let model = cpu.model.as_deref().unwrap_or("unknown");
    let cores = cpu
        .logical_cores
        .map_or_else(|| "unknown".to_owned(), |n| n.to_string());
    format!(
        "{model}; {cores} logical cores; AVX2 {}, FMA {}, NEON {}",
        yes_no(cpu.avx2),
        yes_no(cpu.fma),
        yes_no(cpu.neon)
    )
}

/// One graphics card's line.
pub fn gpu_text(g: &GpuInfo) -> String {
    if g.shared {
        return format!("{}: shares this computer's memory", g.name);
    }
    match (g.vram_free, g.vram_total) {
        (Some(free), Some(total)) => {
            format!(
                "{}: {} free of {} video memory",
                g.name,
                bytes_text(Some(free)),
                bytes_text(Some(total))
            )
        }
        _ => format!("{}: video memory unknown", g.name),
    }
}

/// The audit, one line per item.
pub fn audit(a: &Audit, ui: &mut egui::Ui) {
    let weak = palette_of(ui).text_weak;
    let memory = format!(
        "{} free of {}",
        bytes_text(a.ram_free),
        bytes_text(a.ram_total)
    );
    let disk = match &a.models_dir {
        Some(dir) => format!("{} free in {dir}", bytes_text(a.disk_free)),
        None => "unknown (no models folder)".to_owned(),
    };
    ui.label(RichText::new(format!("Processor: {}", cpu_text(&a.cpu))).color(weak));
    ui.label(RichText::new(format!("Memory: {memory}")).color(weak));
    if a.gpus.is_empty() {
        ui.label(RichText::new("Graphics card: none could be read").color(weak));
    }
    for g in &a.gpus {
        ui.label(RichText::new(format!("Graphics card: {}", gpu_text(g))).color(weak));
    }
    ui.label(RichText::new(format!("Disk for models: {disk}")).color(weak));
}

/// Where a model fits, in words.
pub fn fit_text(fit: Fit) -> &'static str {
    match fit {
        Fit::Video => "fits in video memory",
        Fit::Memory => "fits in memory (runs on the processor)",
        Fit::TooLarge => "too large for this computer",
        Fit::Unknown => "fit unknown",
    }
}

/// One suggestion: name, sizes, fit, whether installed, and the reason for its rank.
pub fn suggestion_line(s: &Suggestion, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("{}. {}", s.rank, s.name)).strong());
        kit::pill(
            ui,
            fit_text(s.fit),
            if studyfrog_service::ollama::fit::fits(s.fit) {
                pal.success
            } else {
                pal.text_weak
            },
        );
        if s.installed {
            kit::pill(ui, "installed", pal.accent);
        }
    });
    ui.label(
        RichText::new(format!(
            "{} {}, download {}, needs {} of memory",
            s.parameters,
            s.quantization,
            bytes_text(s.download_bytes),
            bytes_text(s.memory_bytes)
        ))
        .color(pal.text_weak),
    );
    ui.label(RichText::new(&s.reason).color(pal.text_weak));
}

/// The guide: each step with its commands in copyable blocks; the next step is highlighted.
pub fn guide(steps: &[Step], ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    for (n, step) in steps.iter().enumerate() {
        let title = format!("{}. {}", n + 1, step.title);
        let frame = if step.next {
            egui::Frame::new()
                .stroke(egui::Stroke::new(2.0, pal.accent))
                .inner_margin(8.0)
                .corner_radius(6.0)
        } else {
            egui::Frame::new().inner_margin(8.0)
        };
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong());
                if step.done {
                    kit::pill(ui, "done", pal.success);
                } else if step.next {
                    kit::pill(ui, "next", pal.accent);
                }
            });
            ui.label(RichText::new(&step.note).color(pal.text_weak));
            for cmd in &step.commands {
                command_block(cmd, ui);
            }
        });
    }
}

/// One command in a block with a Copy button.
fn command_block(cmd: &str, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(egui::Label::new(RichText::new(cmd).monospace()).wrap());
        if ui
            .small_button("Copy")
            .on_hover_text("Copies this command to the clipboard. Nothing runs until you run it.")
            .clicked()
        {
            ui.ctx().copy_text(cmd.to_owned());
        }
    });
}
