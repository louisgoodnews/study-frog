//! The plans of the file commands: `import` (the steps of the transfer dialog) and `pdf find` (the PDF
//! viewer's find field). Pure.

use crate::control_plan::Plan;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::transfer_types::Tab;
use crate::transfer_ui::TransferMsg;

fn transfer(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

/// The steps of `import`: open the dialog on the destination, choose a new stack name if any, read the
/// file, then import.
pub fn import(
    path: &str,
    stack: &Option<String>,
    root: bool,
    new_stack: &Option<String>,
) -> Result<Plan, String> {
    let mut open = vec![transfer(TransferMsg::Open(
        Tab::Import,
        if root {
            Some(String::new())
        } else {
            stack.clone()
        },
    ))];
    if let Some(name) = new_stack {
        if name.trim().is_empty() {
            return Err("--new-stack needs a name".to_owned());
        }
        open.push(transfer(TransferMsg::ChooseNewStack));
        open.push(transfer(TransferMsg::NewStackName(name.clone())));
    }
    open.push(transfer(TransferMsg::Path(path.to_owned())));
    open.push(transfer(TransferMsg::Preview));
    Ok(vec![open, vec![transfer(TransferMsg::Import)]])
}

/// The steps of `pdf find`: open the file if one is named, then set the find field.
pub fn pdf_find(file: &Option<String>, text: &str) -> Result<Plan, String> {
    let mut phases = Vec::new();
    if let Some(path) = file {
        phases.push(vec![Msg::Pdf(crate::pdf_types::PdfMsg::Open(
            path.to_owned(),
            0,
        ))]);
    }
    phases.push(vec![Msg::Pdf(crate::pdf_types::PdfMsg::Query(
        text.to_owned(),
    ))]);
    Ok(phases)
}
