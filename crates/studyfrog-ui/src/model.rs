//! The model the app owns: state plus the frame bookkeeping, and the frame function that ties the
//! views to `update`. No threads here, so tests can drive it frame by frame.

use std::time::Duration;

use eframe::egui;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::State;
use crate::update::update;
use crate::view;

/// How often the vault is checked for changes, in seconds of egui time.
pub const TICK_SECONDS: f64 = 1.0;

/// State plus the time of the last tick.
#[derive(Debug, Default)]
pub struct Model {
    /// The UI state.
    pub state: State,
    /// egui time of the last `Tick`.
    pub last_tick: f64,
}

/// Run one message through `update`; returns the effects it asks for.
pub fn apply(model: &mut Model, msg: Msg) -> Vec<Effect> {
    let (state, effects) = update(std::mem::take(&mut model.state), msg);
    model.state = state;
    effects
}

/// Draw one frame: collect the user's messages from the views, add a tick when one is due, and
/// apply everything. Returns the effects to execute.
pub fn frame(model: &mut Model, ui: &mut egui::Ui) -> Vec<Effect> {
    let now = ui.input(|i| i.time);
    model.state.now = now;
    model.state.wall_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    let mut msgs = view::draw(&model.state, ui);
    if ui.input(|i| !i.events.is_empty() || i.pointer.is_moving() || i.pointer.any_down()) {
        msgs.push(Msg::Activity);
    }
    if now - model.last_tick >= TICK_SECONDS {
        model.last_tick = now;
        msgs.push(Msg::Tick);
    }
    ui.ctx()
        .request_repaint_after(Duration::from_secs_f64(TICK_SECONDS));
    msgs.into_iter().flat_map(|m| apply(model, m)).collect()
}
