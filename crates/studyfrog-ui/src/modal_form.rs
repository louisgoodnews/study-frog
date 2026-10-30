//! Small pure helpers of the create dialog's form.

use studyfrog_core::civil::parse_date;
use studyfrog_service::create_more::NewKind;
use studyfrog_service::track::{DEFAULT_DUE_MINUTE, DueDate};

use crate::modal_types::CreateForm;

pub(crate) fn toggled(list: &[u32], minutes: u32) -> Vec<u32> {
    let mut out: Vec<u32> = list.iter().copied().filter(|m| *m != minutes).collect();
    if out.len() == list.len() {
        out.push(minutes);
    }
    out.sort_unstable();
    out
}

pub(crate) fn tags_of(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(crate) fn due_of(form: &CreateForm) -> Option<DueDate> {
    let date = parse_date(form.due.trim()).filter(|_| form.kind == NewKind::Stack)?;
    Some(DueDate {
        date,
        minute: DEFAULT_DUE_MINUTE,
        reminders: form.reminders.clone(),
    })
}
