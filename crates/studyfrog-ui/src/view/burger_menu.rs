//! Floating burger navigation menu prototype (STORY-0147).

use eframe::egui::{self, Pos2, RichText, Sense, vec2};

use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::kit;

const SCREENS: &[(Screen, &str, &str)] = &[
    (Screen::Home, "Home", "Overview & daily queues"),
    (Screen::Library, "Library", "All cards, notes & questions"),
    (Screen::Folders, "Folders", "Vault hierarchy & items"),
    (
        Screen::Rehearse,
        "Rehearse",
        "Active spaced-repetition recall",
    ),
    (
        Screen::Calendar,
        "Calendar",
        "Multi-view schedule & timetable",
    ),
    (Screen::Board, "Board", "Kanban study workflow"),
    (Screen::Reports, "Reports", "Analytics & memory retention"),
    (
        Screen::Prompts,
        "Prompts",
        "Interactive AI content templates",
    ),
    (Screen::Diary, "Diary", "Metacognitive study journal"),
    (Screen::Pdf, "PDF", "Document reader & extracts"),
    (
        Screen::Console,
        "Console",
        "ScriptRunner development workbench",
    ),
    (
        Screen::Achievements,
        "Achievements",
        "Progression & milestones",
    ),
    (Screen::Settings, "Settings", "Preferences & appearance"),
    (Screen::Help, "Help", "Documentation & guides"),
];

/// Draw the floating burger navigation button and popover overlay.
#[allow(deprecated)]
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let popup_id = egui::Id::new("burger-nav-popup-open");
    let mut is_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

    let screen_rect = ui.clip_rect();
    let btn_pos = Pos2::new(16.0, screen_rect.max.y - 56.0);

    egui::Area::new(egui::Id::new("floating-burger-btn"))
        .fixed_pos(btn_pos)
        .order(egui::Order::Foreground)
        .show(ui.ctx(), |ui| {
            let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
            let fill = if resp.hovered() {
                pal.surface_hover
            } else {
                pal.surface
            };
            ui.painter().rect_filled(rect, 8.0, fill);
            ui.painter().rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.0, pal.border),
                egui::StrokeKind::Inside,
            );

            // Three horizontal burger lines
            let cx = rect.center().x;
            let cy = rect.center().y;
            let stroke = egui::Stroke::new(1.8, pal.text);
            ui.painter().line_segment(
                [Pos2::new(cx - 7.0, cy - 5.0), Pos2::new(cx + 7.0, cy - 5.0)],
                stroke,
            );
            ui.painter()
                .line_segment([Pos2::new(cx - 7.0, cy), Pos2::new(cx + 7.0, cy)], stroke);
            ui.painter().line_segment(
                [Pos2::new(cx - 7.0, cy + 5.0), Pos2::new(cx + 7.0, cy + 5.0)],
                stroke,
            );

            if resp.clicked() {
                is_open = !is_open;
                ui.data_mut(|d| d.insert_temp(popup_id, is_open));
            }
            resp.on_hover_text("Navigation menu (Alt+M)");
        });

    if !is_open {
        return;
    }

    // Modal drawer popover
    let popover_pos = Pos2::new(16.0, (screen_rect.max.y - 56.0 - 420.0).max(40.0));
    egui::Area::new(egui::Id::new("floating-burger-popover"))
        .fixed_pos(popover_pos)
        .order(egui::Order::Tooltip)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(pal.surface)
                .stroke(egui::Stroke::new(1.0, pal.border))
                .inner_margin(egui::Margin::symmetric(14, 12))
                .show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Quick Navigation").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if kit::quiet_button(ui, "×").clicked() {
                                is_open = false;
                                ui.data_mut(|d| d.insert_temp(popup_id, false));
                            }
                        });
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(340.0)
                        .show(ui, |ui| {
                            for &(screen, title, desc) in SCREENS {
                                let active = state.screen == screen;
                                let mut text = RichText::new(title).strong();
                                if active {
                                    text = text.color(pal.accent);
                                }
                                ui.horizontal(|ui| {
                                    if ui.selectable_label(active, text).clicked() {
                                        out.push(Msg::Navigate(screen));
                                        is_open = false;
                                        ui.data_mut(|d| d.insert_temp(popup_id, false));
                                    }
                                    ui.label(
                                        RichText::new(desc)
                                            .size(crate::typo::size(ui, crate::typo::Tier::Caption))
                                            .color(pal.text_weak),
                                    );
                                });
                                ui.add_space(2.0);
                            }
                        });
                });
        });
}
