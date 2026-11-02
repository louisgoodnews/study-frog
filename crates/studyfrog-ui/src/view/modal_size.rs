//! The size of the Create dialog and the buttons at the top right and the grip at the lower left that change it.

use eframe::egui::{self, Align, Context, Id, Layout, Role, Sense, WidgetInfo, vec2};
use studyfrog_service::create_more::NewKind;

use crate::modal::{CreateForm, ModalMsg};
use crate::msg::Msg;
use crate::view::icons::{Glyph, icon_button};

pub(super) const NORMAL: f32 = 620.0;
pub(super) const WIDE: f32 = 1000.0;
const MIN_WIDTH: f32 = 480.0;

fn width_id() -> Id {
    Id::unique("create-dialog-width")
}

fn widest(ctx: &Context) -> f32 {
    (ctx.content_rect().width() - 40.0).max(MIN_WIDTH)
}

/// The width of the dialog: what the user dragged it to, else a normal one (wide for a note).
pub fn width(ctx: &Context, kind: NewKind) -> f32 {
    let own = ctx.data(|d| d.get_temp::<f32>(width_id()));
    own.unwrap_or(if kind == NewKind::Note { WIDE } else { NORMAL })
        .clamp(MIN_WIDTH, widest(ctx))
}

fn set_width(ctx: &Context, w: f32) {
    let w = w.clamp(MIN_WIDTH, widest(ctx));
    ctx.data_mut(|d| d.insert_temp(width_id(), w));
}

pub(super) fn window_buttons(ctx: &Context, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if icon_button(ui, Glyph::Close, "Close", false).clicked() {
            out.push(Msg::Modal(ModalMsg::Close));
        }
        let wide = width(ctx, f.kind) > (NORMAL + WIDE) / 2.0;
        let name = if wide {
            "Shrink the dialog"
        } else {
            "Enlarge the dialog"
        };
        if ui.button(name).clicked() {
            set_width(ctx, if wide { NORMAL } else { WIDE });
        }
        ui.menu_button("More", |ui| {
            if ui.button("Import or export…").clicked() {
                let open = crate::transfer_ui::TransferMsg::Open(
                    crate::transfer_ui::Tab::Import,
                    f.parent.clone(),
                );
                out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(open)));
                ui.close();
            }
            if ui.button("Add PDF ...").clicked() {
                let folder = f.parent.clone().unwrap_or_default();
                let add = crate::pdf_library_types::LibraryMsg::AddTo(folder);
                out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Library(add)));
                out.push(Msg::Modal(ModalMsg::Close));
                ui.close();
            }
        });
    });
}

/// A grip at the lower left that widens the dialog when dragged, or with the arrow keys when focused.
pub(super) fn grip(ui: &mut egui::Ui) {
    let (rect, r) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click_and_drag());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "::",
        egui::FontId::monospace(14.0),
        ui.visuals().weak_text_color(),
    );
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Resize the dialog"));
    let key = ui.input(|i| {
        i.key_pressed(egui::Key::ArrowRight) as i8 - i.key_pressed(egui::Key::ArrowLeft) as i8
    });
    let step = if r.has_focus() {
        f32::from(key) * 40.0
    } else {
        0.0
    };
    let drag = if r.dragged() {
        r.drag_delta().x * 2.0
    } else {
        0.0
    };
    if step != 0.0 || drag != 0.0 {
        let now = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(width_id()))
            .unwrap_or(ui.available_width());
        set_width(ui.ctx(), now + step + drag);
    }
}
