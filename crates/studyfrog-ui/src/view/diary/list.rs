//! List view of diary entries with stack and rating filters.

use eframe::egui::{self, RichText, Role, TextEdit, WidgetInfo};
use studyfrog_core::civil::format_date;
use studyfrog_core::diary::STANDARD_SCALES;

use crate::diary_filter::filter_entries;
use crate::diary_msg::DiaryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw the filtered list of diary entries.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);

    filter_controls(state, ui, out);
    ui.add_space(12.0);

    let filtered = filter_entries(
        &state.diary.entries,
        &state.diary.search_query,
        state.diary.filter_stack.as_deref(),
        state.diary.filter_rating_scale.as_deref(),
        state.diary.filter_min_rating,
    );

    if filtered.is_empty() {
        ui.add_space(20.0);
        crate::view::kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("No diary entries match the filter.").color(p.text_weak));
            ui.add_space(8.0);
            let clear_btn = ui.button("Clear filters");
            clear_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Clear filters"));
            if clear_btn.clicked() {
                diary_msg(out, DiaryMsg::SetSearch(String::new()));
                diary_msg(out, DiaryMsg::SetFilterStack(None));
                diary_msg(out, DiaryMsg::SetFilterRating(None, None));
            }
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in filtered {
                crate::view::kit::card(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.heading(&entry.title);
                        ui.label(
                            RichText::new(format_date(entry.date))
                                .strong()
                                .color(p.accent),
                        );
                        if let Some(mins) = entry.study_minutes {
                            crate::view::kit::pill(ui, &format!("{mins} min"), p.accent);
                        }
                        for s in &entry.stacks {
                            crate::view::kit::pill(ui, s, p.text_weak);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let open_btn = ui.button("Open in Editor");
                            open_btn.widget_info(|| {
                                WidgetInfo::labeled(
                                    Role::Button,
                                    true,
                                    format!("Open {} in Markdown editor", entry.title),
                                )
                            });
                            if open_btn.clicked() {
                                out.push(Msg::OpenItem(entry.uid));
                            }

                            let edit_btn = ui.button("Edit");
                            edit_btn.widget_info(|| {
                                WidgetInfo::labeled(
                                    Role::Button,
                                    true,
                                    format!("Edit {}", entry.title),
                                )
                            });
                            if edit_btn.clicked() {
                                diary_msg(out, DiaryMsg::EditEntry(entry.uid));
                            }
                        });
                    });

                    if !entry.ratings.is_empty() {
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            for (k, v) in &entry.ratings {
                                let text = match v {
                                    studyfrog_core::diary::RatingValue::Int(n) => {
                                        format!("{k}: {n}/5")
                                    }
                                    studyfrog_core::diary::RatingValue::Text(t) => {
                                        format!("{k}: {t}")
                                    }
                                };
                                crate::view::kit::pill(ui, &text, p.alt);
                            }
                        });
                    }

                    if !entry.content.trim().is_empty() {
                        ui.add_space(4.0);
                        let preview: String =
                            entry.content.lines().take(2).collect::<Vec<_>>().join(" ");
                        ui.label(RichText::new(preview).italics().color(p.text_weak));
                    }
                });
                ui.add_space(8.0);
            }
        });
}

fn filter_controls(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        let mut search = state.diary.search_query.clone();
        let search_field = ui.add(
            TextEdit::singleline(&mut search)
                .hint_text("Search reflections or tags...")
                .desired_width(220.0),
        );
        search_field.widget_info(|| {
            WidgetInfo::labeled(Role::TextInput, true, "Search reflections or tags")
        });
        if search != state.diary.search_query {
            diary_msg(out, DiaryMsg::SetSearch(search));
        }

        ui.add_space(8.0);
        ui.label("Stack:");
        let current_stack = state.diary.filter_stack.as_deref().unwrap_or("All");
        let stack_combo = egui::ComboBox::from_id_salt("diary-stack-filter")
            .selected_text(current_stack)
            .show_ui(ui, |ui| {
                let all_btn = ui.selectable_label(state.diary.filter_stack.is_none(), "All");
                all_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "All stacks"));
                if all_btn.clicked() {
                    diary_msg(out, DiaryMsg::SetFilterStack(None));
                }
                if let Some(vault) = &state.vault {
                    for s in &vault.overview.stacks {
                        let sel = state.diary.filter_stack.as_deref() == Some(&s.folder);
                        let opt = ui.selectable_label(sel, &s.title);
                        opt.widget_info(|| {
                            WidgetInfo::labeled(
                                Role::Button,
                                true,
                                format!("Filter by {}", s.title),
                            )
                        });
                        if opt.clicked() {
                            diary_msg(out, DiaryMsg::SetFilterStack(Some(s.folder.clone())));
                        }
                    }
                }
            });
        stack_combo
            .response
            .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Filter by stack"));

        ui.add_space(8.0);
        ui.label("Rating:");
        let scale_text = state
            .diary
            .filter_rating_scale
            .as_deref()
            .unwrap_or("Any scale");
        let scale_combo = egui::ComboBox::from_id_salt("diary-scale-filter")
            .selected_text(scale_text)
            .show_ui(ui, |ui| {
                let any_btn =
                    ui.selectable_label(state.diary.filter_rating_scale.is_none(), "Any scale");
                any_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Any scale"));
                if any_btn.clicked() {
                    diary_msg(out, DiaryMsg::SetFilterRating(None, None));
                }
                for &scale in &STANDARD_SCALES {
                    let sel = state.diary.filter_rating_scale.as_deref() == Some(scale);
                    let opt = ui.selectable_label(sel, scale);
                    opt.widget_info(|| {
                        WidgetInfo::labeled(Role::Button, true, format!("Filter scale {scale}"))
                    });
                    if opt.clicked() {
                        let min_r = state.diary.filter_min_rating.unwrap_or(3);
                        diary_msg(
                            out,
                            DiaryMsg::SetFilterRating(Some(scale.to_owned()), Some(min_r)),
                        );
                    }
                }
            });
        scale_combo
            .response
            .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Filter by rating scale"));

        if state.diary.filter_rating_scale.is_some() {
            ui.add_space(4.0);
            ui.label("Min score:");
            for score in 1..=5 {
                let sel = state.diary.filter_min_rating == Some(score);
                let btn = ui.selectable_label(sel, format!("{score}+"));
                btn.widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, format!("Minimum score of {score}"))
                });
                if btn.clicked() {
                    let scale = state.diary.filter_rating_scale.clone();
                    diary_msg(out, DiaryMsg::SetFilterRating(scale, Some(score)));
                }
            }
        }
    });
}
