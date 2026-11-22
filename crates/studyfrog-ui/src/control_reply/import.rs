//! The answer of `import ...`: the preview of the file (or what the last import did), with the pages and
//! files left out and why.

use serde_json::{Value, json};

use crate::state::State;

/// The preview while it is shown, else the summary of the last import; `null` when neither exists yet.
pub fn import_json(state: &State) -> Value {
    if let Some(p) = &state.transfer.preview {
        return json!({
            "adapter": p.adapter,
            "total": p.total,
            "new_stacks": p.new_stacks,
            "left_out": p.warnings,
            "lines": p.lines.iter().take(50).map(|l| json!({"title": l.title, "folder": l.folder})).collect::<Vec<_>>(),
        });
    }
    match &state.transfer.imported {
        Some(s) => json!({
            "created": s.created,
            "replaced": s.replaced,
            "skipped": s.skipped,
            "left_out": s.left_out,
        }),
        None => Value::Null,
    }
}
