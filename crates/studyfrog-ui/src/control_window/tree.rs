//! The window's widget tree. egui builds an AccessKit tree for a frame only while it is asked to, and
//! reports it in the frame's output. A plugin keeps the newest copy for `ui tree`, `ui find` and
//! `ui click`, and hands the clicks the protocol queued to egui as input on its next frame.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use eframe::egui;
use egui::accesskit::{ActionRequest, TreeUpdate};

/// What the plugin keeps between frames.
#[derive(Default)]
pub struct Slot {
    /// Copy the frame's tree when one comes (only while a request waits for it).
    pub want: bool,
    /// Trees copied so far; a request waits for a count above the one it started at.
    pub captures: u64,
    /// The newest copy.
    pub latest: Option<TreeUpdate>,
    /// Clicks to hand to egui on its next frame.
    pub clicks: VecDeque<ActionRequest>,
}

/// The slot, shared by the window's protocol and the plugin.
pub type Shared = Arc<Mutex<Slot>>;

/// A new, empty slot.
pub fn slot_new() -> Shared {
    Arc::new(Mutex::new(Slot::default()))
}

fn lock(s: &Shared) -> MutexGuard<'_, Slot> {
    s.lock().unwrap_or_else(|e| e.into_inner())
}

/// Ask for trees (`true`) or stop (`false`); returns the captures so far.
pub fn want(s: &Shared, on: bool) -> u64 {
    let mut g = lock(s);
    g.want = on;
    g.captures
}

/// Trees copied so far.
pub fn captures(s: &Shared) -> u64 {
    lock(s).captures
}

/// The newest tree copied, if any.
pub fn latest(s: &Shared) -> Option<TreeUpdate> {
    lock(s).latest.clone()
}

/// Queue a click for the next frame.
pub fn queue_click(s: &Shared, req: ActionRequest) {
    lock(s).clicks.push_back(req);
}

fn capture(s: &Shared, update: Option<&TreeUpdate>) {
    let mut g = lock(s);
    if let (true, Some(u)) = (g.want, update) {
        g.latest = Some(u.clone());
        g.captures += 1;
    }
}

fn inject(s: &Shared, events: &mut Vec<egui::Event>) {
    let mut g = lock(s);
    while let Some(req) = g.clicks.pop_front() {
        events.push(egui::Event::AccessKitActionRequest(req));
    }
}

/// The plugin egui calls around each frame: clicks in, the frame's tree out.
pub struct TreePlugin {
    /// The slot it fills.
    pub slot: Shared,
}

impl egui::Plugin for TreePlugin {
    fn debug_name(&self) -> &'static str {
        "studyfrog-control-tree"
    }

    fn input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        inject(&self.slot, &mut input.events);
    }

    fn output_hook(&mut self, _ctx: &egui::Context, output: &mut egui::FullOutput) {
        capture(&self.slot, output.platform_output.accesskit_update.as_ref());
    }
}
