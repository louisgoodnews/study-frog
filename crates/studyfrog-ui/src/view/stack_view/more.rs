//! The stack page's overflow menu and its move submenu.
use super::*;

pub(super) fn more_actions(state: &State, stack: &StackRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("More actions", |ui| {
        if ui
            .button("Memory game")
            .on_hover_text("Plays a memory game with the cards of this stack, with the timing chosen in Rehearse.")
            .clicked()
        {
            out.push(Msg::Navigate(crate::state::Screen::Rehearse));
            out.push(Msg::Memory(crate::setup::memory::MemoryMsg::StartStack(
                stack.folder.clone(),
            )));
            ui.close();
        }
        ui.separator();
        if let Some(pick) = status_menu(ui, stack.status, None) {
            send(out, StackViewMsg::SetStatus(stack.uid, pick));
        }
        if ui
            .button("Details")
            .on_hover_text("Shows the details of this item.")
            .clicked()
        {
            out.push(Msg::OpenItem(stack.uid));
            ui.close();
        }
        if ui
            .button("Select all items (bulk edit)")
            .on_hover_text("Selects every item in the list for a bulk edit.")
            .clicked()
        {
            send(out, StackViewMsg::Select(StackTab::List));
            out.push(Msg::Library(LibraryMsg::Bulk(
                crate::library_bulk::BulkMsg::SelectAll(true),
            )));
            ui.close();
        }
        if ui
            .button("Export…")
            .on_hover_text("Opens the export dialog: choose a format and where to write the file.")
            .clicked()
        {
            let open = crate::transfer_ui::TransferMsg::Open(
                crate::transfer_ui::Tab::Export,
                Some(stack.folder.clone()),
            );
            out.push(Msg::Library(LibraryMsg::Transfer(open)));
            ui.close();
        }
        let ext = &state.extensions;
        if !(ext.card_types.is_empty() && ext.importers.is_empty())
            && ui
                .button("Add from extension…")
                .on_hover_text(
                    "Adds cards from a card type or importer that an extension provides.",
                )
                .clicked()
        {
            out.push(Msg::Modal(ModalMsg::OpenImport));
            ui.close();
        }
        if ui
            .button("Export to PDF…")
            .on_hover_text("Writes this as a PDF file you choose.")
            .clicked()
        {
            out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                crate::pdf_export_types::PdfExportMsg::Open(
                    studyfrog_service::pdf_export::ExportTarget::Stack(stack.folder.clone()),
                ),
            )));
            ui.close();
        }
        if ui
            .button("Make normal folder…")
            .on_hover_text("Turns this stack back into a normal folder.")
            .clicked()
        {
            out.push(Msg::Modal(ModalMsg::AskUnmark(stack.folder.clone())));
            ui.close();
        }
        move_stack_menu(state, stack, ui, out);
    })
    .response
    .on_hover_text("Shows the less common actions.");
}

pub(super) fn move_stack_menu(
    state: &State,
    stack: &StackRow,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let Some(vault) = &state.vault else { return };
    let targets: Vec<_> = vault
        .overview
        .stacks
        .iter()
        .filter(|s| {
            s.folder != stack.folder
                && !studyfrog_service::folder_ops::is_subpath_or_equal(&stack.folder, &s.folder)
                && Some(s.folder.as_str()) != stack.parent.as_deref()
        })
        .collect();
    if stack.parent.is_none() && targets.is_empty() {
        return;
    }
    ui.menu_button("Move stack…", |ui| {
        if stack.parent.is_some()
            && ui
                .button("Make standalone (top-level)")
                .on_hover_text("Moves this stack to the top level.")
                .clicked()
        {
            send(
                out,
                StackViewMsg::MoveStack(stack.folder.clone(), String::new()),
            );
            ui.close();
        }
        for s in targets {
            if ui
                .button(format!("Move under {}", s.title))
                .on_hover_text("Moves this under the chosen stack.")
                .clicked()
            {
                send(
                    out,
                    StackViewMsg::MoveStack(stack.folder.clone(), s.folder.clone()),
                );
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Moves this stack under another one.");
}
