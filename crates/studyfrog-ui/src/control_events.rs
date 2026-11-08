//! The lifecycle event bus of an instance (ADR 0013) and `subscribe [kinds...]` over the control
//! protocol: every event that matches is written as `{"event": "lifecycle", "data": {...}}` on the
//! connection until it closes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::control_logs::{SharedOut, write_line};
use crate::control_reply::{err_line, event_line, ok_line};
use crate::script_events::{EVENT_KINDS, Lifecycle, event_json};

/// The subscribers of an instance: one channel each; closed ones are dropped on the next publish.
pub type EventBus = Arc<Mutex<Vec<Sender<Value>>>>;

/// A bus with no subscribers.
pub fn bus_new() -> EventBus {
    Arc::new(Mutex::new(Vec::new()))
}

/// Send `events` (raised at `at_ms`) to every subscriber still listening.
pub fn publish(bus: &EventBus, events: &[Lifecycle], at_ms: i64) {
    if events.is_empty() {
        return;
    }
    let Ok(mut subs) = bus.lock() else { return };
    for ev in events {
        let value = event_json(ev, at_ms);
        subs.retain(|s| s.send(value.clone()).is_ok());
    }
}

/// A new subscriber's end of the bus.
pub fn subscribe(bus: &EventBus) -> Receiver<Value> {
    let (tx, rx) = channel();
    if let Ok(mut subs) = bus.lock() {
        subs.push(tx);
    }
    rx
}

/// The kinds asked for, checked against the known ones; none means every kind.
pub fn check_kinds(kinds: &[String]) -> Result<Vec<String>, String> {
    if let Some(bad) = kinds.iter().find(|k| !EVENT_KINDS.contains(&k.as_str())) {
        return Err(format!(
            "unknown event {bad} (one of {})",
            EVENT_KINDS.join(", ")
        ));
    }
    Ok(kinds.to_vec())
}

/// Whether an event's JSON is of one of `kinds` (all when empty).
pub fn wanted(event: &Value, kinds: &[String]) -> bool {
    let kind = event.get("kind").and_then(Value::as_str).unwrap_or("");
    kinds.is_empty() || kinds.iter().any(|k| k == kind)
}

/// Write the matching events from `rx` to `out` until the connection is gone or the bus closes.
pub fn forward(rx: &Receiver<Value>, kinds: &[String], out: &SharedOut, alive: &AtomicBool) {
    for event in rx.iter() {
        if !alive.load(Ordering::Relaxed) {
            return;
        }
        if wanted(&event, kinds) && write_line(out, &event_line("lifecycle", event)).is_err() {
            return;
        }
    }
}

/// Answer `subscribe` on a connection: register, then stream on a thread of its own.
pub fn serve_subscribe(
    bus: Option<&EventBus>,
    id: &Value,
    kinds: &[String],
    out: &SharedOut,
    alive: &Arc<AtomicBool>,
) -> String {
    let Some(bus) = bus else {
        return err_line(id, "this instance publishes no events");
    };
    let kinds = match check_kinds(kinds) {
        Ok(k) => k,
        Err(why) => return err_line(id, &why),
    };
    let rx = subscribe(bus);
    let (k, out, alive) = (kinds.clone(), out.clone(), alive.clone());
    std::thread::spawn(move || forward(&rx, &k, &out, &alive));
    let shown: Vec<&str> = if kinds.is_empty() {
        EVENT_KINDS.to_vec()
    } else {
        kinds.iter().map(String::as_str).collect()
    };
    ok_line(
        id,
        serde_json::json!({"subscribed": shown, "following": true}),
    )
}
