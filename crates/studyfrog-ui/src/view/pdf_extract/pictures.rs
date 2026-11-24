//! The pictures section of the PDF extract panel: choose the pictures of the PDF to keep.
use super::*;

pub(super) fn pictures(view: &PdfView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.separator();
    ui.strong("Pictures");
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Save the pictures of this page")
            .on_hover_text("Saves the pictures on this page to files.")
            .clicked()
        {
            send(out, ExtractMsg::SavePictures);
        }
        let snap = egui::Button::selectable(view.extract.snapshot_mode, "Snapshot of a region");
        if ui
            .add(snap)
            .on_hover_text("Then drag a rectangle on the page")
            .clicked()
        {
            send(out, ExtractMsg::ToggleSnapshot);
        }
    });
    for p in &view.extract.pictures {
        ui.horizontal(|ui| {
            ui.label(p);
            let make = crate::recall_types::RecallMsg::OcclusionCard(
                view.extract.folder.clone(),
                p.clone(),
            );
            if ui
                .small_button(format!("Occlusion card from {p}"))
                .on_hover_text("Makes a picture-mask card from this part of the page.")
                .clicked()
            {
                out.push(Msg::Recall(make));
            }
            if ui
                .small_button("Copy Markdown")
                .on_hover_text("Copies this as Markdown text.")
                .clicked()
            {
                let name = p.rsplit('/').next().unwrap_or(p);
                ui.ctx().copy_text(format!(
                    "![{}]({p})",
                    name.rsplit_once('.').map_or(name, |(s, _)| s)
                ));
            }
        });
    }
    let mut include = view.extract.include_pictures;
    let none = view.extract.pictures.is_empty();
    ui.add_enabled(
        !none,
        egui::Checkbox::new(&mut include, "Put these pictures at the end of the note"),
    );
    if include != view.extract.include_pictures {
        send(out, ExtractMsg::IncludePictures(include));
    }
}
