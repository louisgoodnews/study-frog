//! The session form: its data, edits, and the validated conversion to a calendar [`Session`].
//! Pure: no egui, no I/O.

use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{
    Date, LocalTime, format_date, format_minute, parse_date, parse_minute,
};
use studyfrog_core::uid::Uid;

/// The reminder lead times the forms offer, in minutes before the start.
pub const REMINDER_CHOICES: [(u32, &str); 5] = [
    (0, "At the time"),
    (15, "15 minutes before"),
    (60, "1 hour before"),
    (1440, "1 day before"),
    (10080, "1 week before"),
];

/// How the session repeats, without the numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatKind {
    /// Once.
    Never,
    /// Every n days.
    Daily,
    /// Every n weeks on chosen weekdays.
    Weekly,
    /// Every n months.
    Monthly,
}

/// When the repetition stops, without the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndsKind {
    /// Never.
    Forever,
    /// On a date.
    On,
    /// After a number of occurrences.
    After,
}

/// The fields as the user edits them (text stays text until it is validated).
#[derive(Debug, Clone, PartialEq)]
pub struct SessionForm {
    /// The session being edited; `None` for a new one.
    pub editing: Option<Uid>,
    /// Title.
    pub title: String,
    /// First date, `YYYY-MM-DD`.
    pub date: String,
    /// Start time, `HH:MM`.
    pub time: String,
    /// Length in minutes.
    pub duration: String,
    /// The stack to rehearse.
    pub stack: Option<Uid>,
    /// Repeat rule.
    pub repeat: RepeatKind,
    /// Interval as typed.
    pub every: String,
    /// Weekday bit mask (bit 0 = Monday).
    pub days: u8,
    /// End rule.
    pub ends: EndsKind,
    /// End date or count as typed.
    pub ends_value: String,
    /// Notes.
    pub notes: String,
    /// What kind of entry it is.
    pub kind: EventKind,
    /// Reminder lead times in minutes.
    pub reminders: Vec<u32>,
    /// Dates already cancelled (kept when editing).
    pub cancelled: Vec<Date>,
}

/// One change to the form.
#[derive(Debug, Clone, PartialEq)]
pub enum FormEdit {
    /// The title field.
    Title(String),
    /// The date field.
    Date(String),
    /// The time field.
    Time(String),
    /// The duration field.
    Duration(String),
    /// The stack choice.
    Stack(Option<Uid>),
    /// The repeat choice.
    Repeat(RepeatKind),
    /// The interval field.
    Every(String),
    /// A weekday (0 = Monday) on or off.
    Day(u8, bool),
    /// The end choice.
    Ends(EndsKind),
    /// The end value field.
    EndsValue(String),
    /// The notes field.
    Notes(String),
    /// The kind of entry.
    Kind(EventKind),
    /// A reminder lead time (minutes) on or off.
    Reminder(u32),
}

/// A blank form for a session starting on `date`.
pub fn form_new(date: Date) -> SessionForm {
    SessionForm {
        editing: None,
        title: String::new(),
        date: format_date(date),
        time: "09:00".to_owned(),
        duration: "30".to_owned(),
        stack: None,
        repeat: RepeatKind::Never,
        every: "1".to_owned(),
        days: 0,
        ends: EndsKind::Forever,
        ends_value: String::new(),
        notes: String::new(),
        kind: EventKind::default(),
        reminders: vec![15],
        cancelled: Vec::new(),
    }
}

/// The form showing an existing session.
pub fn form_from(s: &Session) -> SessionForm {
    let (repeat, every, days) = match s.repeat {
        Repeat::Never => (RepeatKind::Never, 1, 0),
        Repeat::Daily { every } => (RepeatKind::Daily, every, 0),
        Repeat::Weekly { every, days } => (RepeatKind::Weekly, every, days),
        Repeat::Monthly { every } => (RepeatKind::Monthly, every, 0),
    };
    let (ends, ends_value) = match s.ends {
        Ends::Forever => (EndsKind::Forever, String::new()),
        Ends::On(d) => (EndsKind::On, format_date(d)),
        Ends::After(n) => (EndsKind::After, n.to_string()),
    };
    SessionForm {
        editing: Some(s.id),
        title: s.title.clone(),
        date: format_date(s.start.date),
        time: format_minute(s.start.minute),
        duration: s.duration_min.to_string(),
        stack: s.stack,
        repeat,
        every: every.to_string(),
        days,
        ends,
        ends_value,
        notes: s.notes.clone(),
        kind: s.kind,
        reminders: s.reminders.clone(),
        cancelled: s.cancelled.clone(),
    }
}

/// Apply one edit.
pub fn apply_form_edit(f: SessionForm, edit: FormEdit) -> SessionForm {
    match edit {
        FormEdit::Title(title) => SessionForm { title, ..f },
        FormEdit::Date(date) => SessionForm { date, ..f },
        FormEdit::Time(time) => SessionForm { time, ..f },
        FormEdit::Duration(duration) => SessionForm { duration, ..f },
        FormEdit::Stack(stack) => SessionForm { stack, ..f },
        FormEdit::Repeat(repeat) => SessionForm { repeat, ..f },
        FormEdit::Every(every) => SessionForm { every, ..f },
        FormEdit::Day(i, on) => SessionForm {
            days: if on {
                f.days | (1 << i)
            } else {
                f.days & !(1 << i)
            },
            ..f
        },
        FormEdit::Ends(ends) => SessionForm { ends, ..f },
        FormEdit::EndsValue(ends_value) => SessionForm { ends_value, ..f },
        FormEdit::Notes(notes) => SessionForm { notes, ..f },
        FormEdit::Kind(kind) => SessionForm { kind, ..f },
        FormEdit::Reminder(m) => SessionForm {
            reminders: crate::modal_form::toggled(&f.reminders, m),
            ..f
        },
    }
}

fn number<T: std::str::FromStr>(text: &str, what: &str) -> Result<T, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{what} must be a whole number"))
}

fn repeat_of(f: &SessionForm) -> Result<Repeat, String> {
    let every = || number::<u16>(&f.every, "Repeat interval");
    Ok(match f.repeat {
        RepeatKind::Never => Repeat::Never,
        RepeatKind::Daily => Repeat::Daily { every: every()? },
        RepeatKind::Weekly => Repeat::Weekly {
            every: every()?,
            days: f.days,
        },
        RepeatKind::Monthly => Repeat::Monthly { every: every()? },
    })
}

fn ends_of(f: &SessionForm) -> Result<Ends, String> {
    Ok(match f.ends {
        EndsKind::Forever => Ends::Forever,
        EndsKind::On => Ends::On(
            parse_date(f.ends_value.trim()).ok_or("The end date must look like 2026-12-31")?,
        ),
        EndsKind::After => Ends::After(number(&f.ends_value, "The number of occurrences")?),
    })
}

/// The session the form describes; a new one gets `Uid(0)` and the executor assigns its id.
pub fn session_from_form(f: &SessionForm) -> Result<Session, String> {
    let date = parse_date(f.date.trim()).ok_or("The date must look like 2026-12-31")?;
    let minute = parse_minute(f.time.trim()).ok_or("The time must look like 09:30")?;
    let repeat = if f.repeat == RepeatKind::Never {
        Repeat::Never
    } else {
        repeat_of(f)?
    };
    Ok(Session {
        id: f.editing.unwrap_or(Uid(0)),
        title: f.title.trim().to_owned(),
        stack: f.stack,
        start: LocalTime { date, minute },
        duration_min: number(&f.duration, "The duration")?,
        repeat,
        ends: if f.repeat == RepeatKind::Never {
            Ends::Forever
        } else {
            ends_of(f)?
        },
        cancelled: f.cancelled.clone(),
        notes: f.notes.clone(),
        kind: f.kind,
        reminders: f.reminders.clone(),
    })
}
