//! The Console's glue in the window (STORY-0133): its lines go to the window's own control protocol
//! (on a thread, since the window answers them in later frames), `script run|list` go to the script
//! runner, and every answer, followed log line and lifecycle event comes back through one channel.

use std::io::{self, Write};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::console_ui::ConsoleMsg;
use crate::control_conn::{ConnCtx, answer};
use crate::control_logs::SharedOut;
use crate::msg::Msg;

/// What the Console talks to.
pub struct ConsoleLink {
    /// The window's control protocol, when it is served.
    pub ctx: Option<ConnCtx>,
    /// Where answers and events are written; each line arrives in `lines`.
    pub out: SharedOut,
    /// Lines coming back.
    pub lines: Receiver<String>,
    /// The Console's connection is open (for followed logs and events).
    pub alive: Arc<AtomicBool>,
    /// The script runner of this window.
    #[cfg(unix)]
    pub runner: Option<crate::script_runner::Runner>,
}

/// A writer that hands each complete line to a channel.
pub struct LineSink {
    /// Where lines go.
    pub tx: Sender<String>,
    /// What came after the last line break.
    pub buf: Vec<u8>,
    /// Called after each line (the window repaints).
    pub wake: Option<crate::control_conn::Wake>,
}

fn sink_write(s: &mut LineSink, data: &[u8]) -> io::Result<usize> {
    s.buf.extend_from_slice(data);
    while let Some(end) = s.buf.iter().position(|b| *b == b'\n') {
        let line: Vec<u8> = s.buf.drain(..=end).collect();
        let text = String::from_utf8_lossy(&line).trim_end().to_owned();
        s.tx.send(text)
            .map_err(|_| io::Error::other("the console is gone"))?;
        if let Some(wake) = &s.wake {
            wake();
        }
    }
    Ok(data.len())
}

impl Write for LineSink {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        sink_write(self, data)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A link to `ctx` (none: the Console can only say so) that already receives every lifecycle event.
pub fn link_new(
    ctx: Option<ConnCtx>,
    #[cfg(unix)] runner: Option<crate::script_runner::Runner>,
) -> ConsoleLink {
    let (tx, lines) = channel();
    let wake = ctx.as_ref().map(|c| c.wake.clone());
    let out: SharedOut = Arc::new(Mutex::new(Box::new(LineSink {
        tx,
        buf: Vec::new(),
        wake,
    })));
    let alive = Arc::new(AtomicBool::new(true));
    if let Some(c) = &ctx {
        let _ = answer(c, "subscribe", &out, &alive);
    }
    ConsoleLink {
        ctx,
        out,
        lines,
        alive,
        #[cfg(unix)]
        runner,
    }
}

fn say(link: &ConsoleLink, line: &str) {
    let _ = crate::control_logs::write_line(&link.out, line);
}

#[cfg(unix)]
fn script(link: &ConsoleLink, words: &[String]) {
    let Some(r) = &link.runner else {
        return say(link, &json!({"ok": false, "error": "scripts are not run in this window (no application home or no control socket)"}).to_string());
    };
    match words {
        [cmd, what, args @ ..] if cmd == "run" => {
            match crate::script_runner::run_named(r, what, args) {
                Ok(job) => say(
                    link,
                    &format!("started {job}; its output follows when it ends"),
                ),
                Err(e) => say(link, &json!({"ok": false, "error": e}).to_string()),
            }
        }
        [cmd] if cmd == "list" => say(link, &crate::script_runner::list_text(&r.paths).0),
        _ => say(
            link,
            &json!({"ok": false, "error": "script run JOB|FILE [ARGS] | script list"}).to_string(),
        ),
    }
}

#[cfg(not(unix))]
fn script(link: &ConsoleLink, _: &[String]) {
    say(
        link,
        &json!({"ok": false, "error": "scripts need a Unix system"}).to_string(),
    );
}

/// Send one typed line.
pub fn submit(link: &ConsoleLink, line: &str) {
    let words = crate::control_parse::tokenize(line).unwrap_or_default();
    if words.first().map(String::as_str) == Some("script") {
        return script(link, &words[1..]);
    }
    let Some(ctx) = link.ctx.clone() else {
        return say(link, &json!({"ok": false, "error": "the console needs the control protocol (start without --no-control)"}).to_string());
    };
    let (out, alive, line) = (link.out.clone(), link.alive.clone(), line.to_owned());
    std::thread::spawn(move || {
        let text = answer(&ctx, &line, &out, &alive);
        let _ = crate::control_logs::write_line(&out, &text);
    });
}

/// The lines that came back, as messages.
pub fn poll(link: &ConsoleLink) -> Vec<Msg> {
    link.lines
        .try_iter()
        .map(|l| Msg::Console(ConsoleMsg::Line(l)))
        .collect()
}
