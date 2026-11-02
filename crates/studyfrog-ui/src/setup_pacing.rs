//! The pacing of a run in the setup: free, a countdown with its minutes, or a count up. Pure.

use studyfrog_core::pacing::{MAX_COUNTDOWN_SECS, MIN_COUNTDOWN_SECS, Pacing};

/// Which pacing is chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacingKind {
    /// No clock.
    Free,
    /// The run ends when its time is up.
    Countdown,
    /// The elapsed time is shown.
    CountUp,
}

/// One change of the pacing fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacingEdit {
    /// Choose the pacing.
    Kind(PacingKind),
    /// The minutes of a countdown.
    Minutes(String),
}

/// The fields after a change.
pub fn edited(f: PacingForm, e: PacingEdit) -> PacingForm {
    match e {
        PacingEdit::Kind(kind) => PacingForm { kind, ..f },
        PacingEdit::Minutes(minutes) => PacingForm { minutes, ..f },
    }
}

/// The countdown presets, in minutes.
pub const PRESET_MINUTES: [u32; 4] = [5, 10, 15, 25];

/// The pacing fields as the user edits them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacingForm {
    /// The choice.
    pub kind: PacingKind,
    /// The minutes of a countdown, as typed.
    pub minutes: String,
}

impl Default for PacingForm {
    fn default() -> Self {
        PacingForm {
            kind: PacingKind::Free,
            minutes: "10".to_owned(),
        }
    }
}

/// The form for a plan's pacing.
pub fn form_of(p: Pacing) -> PacingForm {
    match p {
        Pacing::Free => PacingForm::default(),
        Pacing::CountUp => PacingForm {
            kind: PacingKind::CountUp,
            ..PacingForm::default()
        },
        Pacing::Countdown(s) => PacingForm {
            kind: PacingKind::Countdown,
            minutes: (s / 60).to_string(),
        },
    }
}

/// The pacing the form asks for, or why it cannot be had.
pub fn pacing_from(f: &PacingForm) -> Result<Pacing, String> {
    match f.kind {
        PacingKind::Free => Ok(Pacing::Free),
        PacingKind::CountUp => Ok(Pacing::CountUp),
        PacingKind::Countdown => {
            let minutes: u32 = f
                .minutes
                .trim()
                .parse()
                .map_err(|_| "the minutes of a countdown must be a whole number".to_owned())?;
            let secs = minutes.saturating_mul(60);
            if (MIN_COUNTDOWN_SECS..=MAX_COUNTDOWN_SECS).contains(&secs) {
                Ok(Pacing::Countdown(secs))
            } else {
                Err(format!(
                    "a countdown runs from {} to {} minutes",
                    MIN_COUNTDOWN_SECS / 60,
                    MAX_COUNTDOWN_SECS / 60
                ))
            }
        }
    }
}
