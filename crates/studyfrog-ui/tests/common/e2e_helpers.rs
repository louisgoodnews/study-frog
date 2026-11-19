#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]
//! Shared opaque-box E2E harness, `Host` picker stubs, TEA loop, input, and AccessKit helpers.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use eframe::egui::accesskit::Role;
use eframe::egui::{Key, Modifiers, Rect, vec2};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::{Harness, Node};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, execute, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::model::{Model, apply, frame};
use studyfrog_ui::msg::Msg;

use super::views::App;
use super::{NOW_NS, TempDir, run_to_idle, sample_vault, temp_dir};

thread_local! {
    static PICKED_FILE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static PICKED_FOLDER: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Set the path returned by the thread-local stub `Host::file_picker`.
pub fn set_picked_file(path: Option<PathBuf>) {
    PICKED_FILE.with(|cell| *cell.borrow_mut() = path);
}

/// Set the path returned by the thread-local stub `Host::folder_picker`.
pub fn set_picked_folder(path: Option<PathBuf>) {
    PICKED_FOLDER.with(|cell| *cell.borrow_mut() = path);
}

/// Clear both thread-local stub pickers.
pub fn clear_pickers() {
    set_picked_file(None);
    set_picked_folder(None);
}

/// Non-capturing `Host::file_picker` callback reading the thread-local picked file.
pub fn stub_file_picker(_extensions: &[&str]) -> Option<PathBuf> {
    PICKED_FILE.with(|cell| cell.borrow().clone())
}

/// Non-capturing `Host::folder_picker` callback reading the thread-local picked folder.
pub fn stub_folder_picker() -> Option<PathBuf> {
    PICKED_FOLDER.with(|cell| cell.borrow().clone())
}

/// Construct a `Host` wired with thread-local file/folder picker stubs.
pub fn host_with_stub_pickers() -> Host {
    Host {
        file_picker: Some(stub_file_picker),
        folder_picker: Some(stub_folder_picker),
        ..Host::default()
    }
}

/// Boot a `Worker` + `Harness` on an existing vault directory with a custom `Host`.
pub fn boot_harness_on(
    vault_path: &Path,
    config_dir: Option<PathBuf>,
    host: Host,
) -> (Worker, Harness<'static, App>) {
    let mut worker = worker_with_host(config_dir, None, host);
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let vault_str = vault_path.display().to_string();
    run_to_idle(&mut model, &mut worker, vec![Effect::OpenVault(vault_str)]);
    assert!(model.state.vault.is_some(), "vault failed to open");
    let mut h = Harness::builder()
        .with_size(vec2(1280.0, 800.0))
        .build_ui_state(
            |ui, (model, effects): &mut App| effects.extend(frame(model, ui)),
            (model, Vec::new()),
        );
    run_harness_until_idle(&mut h, &mut worker);
    (worker, h)
}

/// Boot an isolated `sample_vault` with thread-local `Host` picker stubs.
pub fn boot_sample_harness(label: &str) -> (TempDir, Worker, Harness<'static, App>) {
    clear_pickers();
    let dir = sample_vault(label);
    let cfg = dir.path.join(".e2e-config");
    let (worker, h) = boot_harness_on(&dir.path, Some(cfg), host_with_stub_pickers());
    (dir, worker, h)
}

/// Boot an isolated empty `TempDir` vault with thread-local `Host` picker stubs.
pub fn boot_empty_harness(label: &str) -> (TempDir, Worker, Harness<'static, App>) {
    clear_pickers();
    let dir = temp_dir(label);
    let cfg = dir.path.join(".e2e-config");
    let (worker, h) = boot_harness_on(&dir.path, Some(cfg), host_with_stub_pickers());
    (dir, worker, h)
}

/// Step one frame and drain all resulting TEA effects through `worker` at `now_ns`.
pub fn step_with_worker(h: &mut Harness<'_, App>, worker: &mut Worker, now_ns: i64) -> usize {
    h.step();
    let mut drained = std::mem::take(&mut h.state_mut().1);
    let count = drained.len();
    for _ in 0..20 {
        if drained.is_empty() {
            return count;
        }
        let msgs: Vec<Msg> = drained
            .drain(..)
            .flat_map(|e| execute(worker, e, now_ns))
            .collect();
        let next: Vec<Effect> = msgs
            .into_iter()
            .flat_map(|m| apply(&mut h.state_mut().0, m))
            .collect();
        drained = next;
    }
    panic!("harness effects did not settle");
}

/// Step frames and drain TEA effects until a rendered frame emits zero effects.
pub fn run_harness_until_idle(h: &mut Harness<'_, App>, worker: &mut Worker) {
    for _ in 0..12 {
        if step_with_worker(h, worker, NOW_NS) == 0 {
            return;
        }
    }
}

/// Advance `ticks` seconds of harness time (`4` steps of `0.25s` per `TICK_SECONDS`).
pub fn advance_ticks(h: &mut Harness<'_, App>, worker: &mut Worker, ticks: usize) {
    let steps = ticks * 4;
    for _ in 0..steps {
        step_with_worker(h, worker, NOW_NS);
    }
}

/// Press a single key and settle resulting frames and effects.
pub fn press_key(h: &mut Harness<'_, App>, worker: &mut Worker, key: Key) {
    h.key_press(key);
    run_harness_until_idle(h, worker);
}

/// Press a key with keyboard modifiers and settle resulting frames and effects.
pub fn press_shortcut(
    h: &mut Harness<'_, App>,
    worker: &mut Worker,
    modifiers: Modifiers,
    key: Key,
) {
    h.key_press_modifiers(modifiers, key);
    run_harness_until_idle(h, worker);
}

fn node_by_label<'a>(h: &'a Harness<'_, App>, label: &'a str) -> Node<'a> {
    h.query_all_by_role_and_label(Role::Button, label)
        .next()
        .or_else(|| h.query_all_by_label(label).next())
        .unwrap_or_else(|| panic!("no accessible node found with label {label:?}"))
}

/// Click the accessible node matching `label` and settle frames and effects.
pub fn click_label(h: &mut Harness<'_, App>, worker: &mut Worker, label: &str) {
    node_by_label(h, label).click();
    run_harness_until_idle(h, worker);
}

/// Hover over `label` and step `>= 5` frames (`1.25s > 0.5s` `tooltip_delay`) so tooltips render.
pub fn hover_for_tooltip(h: &mut Harness<'_, App>, worker: &mut Worker, label: &str) {
    node_by_label(h, label).hover();
    for _ in 0..5 {
        step_with_worker(h, worker, NOW_NS);
    }
}

/// Find the first AccessKit node whose label or value contains `substr`.
pub fn find_by_label_contains<'a>(h: &'a Harness<'_, App>, substr: &'a str) -> Option<Node<'a>> {
    h.query_all_by_label_contains(substr).next()
}

/// Collect `(label, rect)` for every `Role::Button` whose vertical center lies in `[min_y, max_y]`.
pub fn button_rects_in_band(h: &Harness<'_, App>, min_y: f32, max_y: f32) -> Vec<(String, Rect)> {
    h.query_all_by_role(Role::Button)
        .filter_map(|node| {
            let rect = node.rect();
            let cy = rect.center().y;
            if cy >= min_y && cy <= max_y {
                let label = node.accesskit_node().label().unwrap_or_default().to_owned();
                Some((label, rect))
            } else {
                None
            }
        })
        .collect()
}

/// Count `Role::Button` widgets whose vertical center lies within `[min_y, max_y]`.
pub fn count_buttons_in_band(h: &Harness<'_, App>, min_y: f32, max_y: f32) -> usize {
    button_rects_in_band(h, min_y, max_y).len()
}

/// Return `true` when two rectangles overlap in positive interior area.
pub fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
}

/// Return `true` when `inner` fits completely inside `bounds` (within `0.5` pt tolerance).
pub fn rect_inside_bounds(inner: Rect, bounds: Rect) -> bool {
    const EPS: f32 = 0.5;
    inner.min.x >= bounds.min.x - EPS
        && inner.min.y >= bounds.min.y - EPS
        && inner.max.x <= bounds.max.x + EPS
        && inner.max.y <= bounds.max.y + EPS
}
