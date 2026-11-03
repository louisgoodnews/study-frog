//! The question before leaving with unsaved changes in the editor.

use eframe::egui;

use crate::chrome::ChromeMsg;
use crate::msg::Msg;
use crate::quit_ui::QuitMsg;
use crate::state::State;

fn send(out: &mut Vec<Msg>, m: QuitMsg) {
    out.push(Msg::Chrome(ChromeMsg::Quit(m)));
}

/// 'Exit with unsaved changes?', when Exit was pressed while the editor holds some.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !state.chrome.confirm_quit {
        return;
    }
    let response = egui::Modal::new(egui::Id::unique("quit-question")).show(ui.ctx(), |ui| {
        ui.set_width(380.0);
        ui.heading("Exit StudyFrog?");
        ui.label("The document you are editing has changes that are not saved. They will be lost.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Keep working").clicked() {
                send(out, QuitMsg::Cancel);
            }
            if ui.button("Exit anyway").clicked() {
                send(out, QuitMsg::Confirm);
            }
        });
    });
    if response.should_close() {
        send(out, QuitMsg::Cancel);
    }
}
