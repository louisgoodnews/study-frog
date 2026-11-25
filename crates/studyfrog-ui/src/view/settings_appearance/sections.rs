//! The sample and the defaults sections of the appearance settings.
use super::*;

pub(in crate::view) fn sample_section(_: &State, ui: &mut egui::Ui, _: &mut Vec<Msg>) {
    group(ui, "Sample", |ui| {
        let p = palette_of(ui);
        ui.label(
            RichText::new("A heading")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.label("Body text with a link:");
        ui.hyperlink_to("a link", "https://example.org");
        ui.horizontal(|ui| {
            let _ = ui
                .button("A button")
                .on_hover_text("A sample button, to show the look.");
            let _ = kit::primary_button(ui, "The main action")
                .on_hover_text("A sample button, to show the look.");
            kit::pill(ui, "a status", p.success);
            kit::pill(ui, "a warning", p.warning);
            kit::pill(ui, "an error", p.danger);
        });
        kit::progress_with_text(ui, 0.4, "40 % of the sample", "Sample progress");
        ui.weak("Weak text for hints.");
    });
}

pub(in crate::view) fn defaults_section(_: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(6.0);
    if ui
        .button("Restore the default appearance")
        .on_hover_text("Puts the colours, density and text size back to the defaults.")
        .clicked()
    {
        send(out, AppearanceMsg::Defaults);
    }
}
