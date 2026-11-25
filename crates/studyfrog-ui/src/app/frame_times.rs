//! Opt-in frame-time capture (`--frame-times FILE`).
//!
//! The recorder is data: a bounded list of samples, the CSV text of them and the summary statistics.
//! [`csv`] and [`summary`] are pure, so they are tested without a window. Only [`finish`] touches the disk.
//! Off by default: the window keeps no recorder unless the flag is given.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::path::PathBuf;

/// The most samples kept (about 32 MB). Frames drawn after that are counted, not stored.
pub const MAX_SAMPLES: usize = 1_000_000;

/// One drawn frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// Frame number, counted from 0 over the stored frames.
    pub frame: u64,
    /// The screen showing after the frame was drawn.
    pub screen: &'static str,
    /// Time the app spent on the frame, in microseconds.
    pub frame_us: u64,
}

/// The samples of one capture, and where they are written on exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorder {
    /// The CSV file written by [`finish`].
    pub path: PathBuf,
    /// The stored samples, in drawing order.
    pub samples: Vec<Sample>,
    /// Frames drawn after the list was full.
    pub dropped: u64,
}

/// Median, 95th percentile, worst frame and frame count of a capture, in microseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    /// Number of frames summarised.
    pub frames: usize,
    /// Median frame time (mean of the two middle frames for an even count).
    pub median_us: u64,
    /// 95th percentile frame time (nearest rank).
    pub p95_us: u64,
    /// The slowest frame.
    pub worst_us: u64,
}

/// A recorder that will write `path` when it is finished.
pub fn start(path: PathBuf) -> Recorder {
    studyfrog_log::log_info!("frame_times", "frame-time capture started"; path = path.display().to_string());
    Recorder {
        path,
        samples: Vec::new(),
        dropped: 0,
    }
}

/// Record one drawn frame. Past [`MAX_SAMPLES`] the frame is counted in `dropped` instead.
pub fn push(rec: &mut Recorder, screen: &'static str, frame_us: u64) {
    if rec.samples.len() >= MAX_SAMPLES {
        rec.dropped = rec.dropped.saturating_add(1);
        if rec.dropped == 1 {
            studyfrog_log::log_warn!(
                "frame_times",
                "sample list full at {} frames; later frames are counted, not stored",
                MAX_SAMPLES
            );
        }
        return;
    }
    let frame = u64::try_from(rec.samples.len()).unwrap_or(u64::MAX);
    rec.samples.push(Sample {
        frame,
        screen,
        frame_us,
    });
}

/// The CSV text: a header line `frame,screen,frame_us`, then one line per sample.
pub fn csv(samples: &[Sample]) -> String {
    let mut out = String::from("frame,screen,frame_us\n");
    for s in samples {
        // Writing into a String cannot fail.
        let _ = writeln!(out, "{},{},{}", s.frame, field(s.screen), s.frame_us);
    }
    out
}

/// A CSV field, quoted (with doubled quotes) when it holds a comma, a quote or a line break.
fn field(text: &str) -> Cow<'_, str> {
    if text.contains([',', '"', '\n', '\r']) {
        Cow::Owned(format!("\"{}\"", text.replace('"', "\"\"")))
    } else {
        Cow::Borrowed(text)
    }
}

/// Median, p95, worst and count of the frame times, or `None` when there are none.
/// The durations are given; nothing here reads a clock.
pub fn summary(durations_us: &[u64]) -> Option<Summary> {
    if durations_us.is_empty() {
        return None;
    }
    let mut sorted = durations_us.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let median_us = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        sorted[n / 2 - 1].saturating_add(sorted[n / 2]) / 2
    };
    // Nearest rank: the smallest value with at least 95 % of the frames at or below it.
    let rank = (n * 95).div_ceil(100);
    Some(Summary {
        frames: n,
        median_us,
        p95_us: sorted[rank - 1],
        worst_us: sorted[n - 1],
    })
}

/// Write the CSV to the recorder's path and log the summary. A failed write is logged, not returned:
/// the window is closing and has nowhere to report it. The file is written even when no frame was drawn
/// (it then holds the header only).
pub fn finish(rec: Recorder) {
    let _timing = studyfrog_debug::timed("frame_times", "write_csv");
    studyfrog_debug::check(
        "frame_times",
        !rec.samples.is_empty(),
        "no frames were drawn while capturing",
    );
    let path_text = rec.path.display().to_string();
    let durations: Vec<u64> = rec.samples.iter().map(|s| s.frame_us).collect();
    match std::fs::write(&rec.path, csv(&rec.samples)) {
        Ok(()) => match summary(&durations) {
            Some(s) => studyfrog_log::log_info!(
                "frame_times",
                "wrote {} frames to {}",
                s.frames,
                path_text;
                median_us = s.median_us,
                p95_us = s.p95_us,
                worst_us = s.worst_us,
                dropped = rec.dropped
            ),
            None => studyfrog_log::log_warn!(
                "frame_times",
                "no frames drawn; wrote the header only to {}",
                path_text
            ),
        },
        Err(e) => studyfrog_log::log_error!(
            "frame_times",
            "could not write frame times to {}: {}",
            path_text,
            e
        ),
    }
    studyfrog_debug::checkpoint("frame_times", "capture finished");
}
