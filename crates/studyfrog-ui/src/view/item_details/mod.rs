//! The side column of the item page: a tabbed notebook view organizing item details
//! into Properties, Schedule, Relations, and Learning tabs.

mod tab_learning;
mod tab_properties;
mod tab_relations;
mod tab_schedule;

use eframe::egui::{self, Role, WidgetInfo};
use studyfrog_md::doc::Doc;

use crate::editor::Editor;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit;

/// The tabs of the item details notebook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailsTab {
    #[default]
    Properties,
    Schedule,
    Relations,
    Learning,
}

const STACK_TABS: &[DetailsTab] = &[
    DetailsTab::Properties,
    DetailsTab::Schedule,
    DetailsTab::Relations,
];

const ITEM_TABS: &[DetailsTab] = &[
    DetailsTab::Properties,
    DetailsTab::Relations,
    DetailsTab::Learning,
];

fn tabs_of(doc: &Doc) -> &'static [DetailsTab] {
    if matches!(doc, Doc::Stack(_)) {
        STACK_TABS
    } else {
        ITEM_TABS
    }
}

fn tab_label(tab: DetailsTab) -> &'static str {
    match tab {
        DetailsTab::Properties => "Properties",
        DetailsTab::Schedule => "Schedule",
        DetailsTab::Relations => "Relations",
        DetailsTab::Learning => "Learning",
    }
}

fn tab_bar(e: &Editor, ui: &mut egui::Ui) -> DetailsTab {
    let tabs = tabs_of(&e.doc);
    let id = ui.make_persistent_id("item_details_tab");
    let stored = ui
        .data(|d| d.get_temp::<DetailsTab>(id))
        .unwrap_or_default();
    let current = if tabs.contains(&stored) {
        stored
    } else {
        tabs[0]
    };

    let mut chosen = current;
    ui.horizontal(|ui| {
        for &tab in tabs {
            let selected = tab == current;
            let resp = ui.selectable_label(selected, tab_label(tab));
            resp.widget_info(|| WidgetInfo::labeled(Role::Tab, selected, tab_label(tab)));
            if resp.clicked() {
                chosen = tab;
                ui.data_mut(|d| d.insert_temp(id, tab));
            }
        }
    });
    chosen
}

/// The details view entry point for the item editor side column.
pub fn details(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Details");
    let current = tab_bar(e, ui);
    ui.add_space(8.0);
    match current {
        DetailsTab::Properties => tab_properties::draw(state, e, ui, out),
        DetailsTab::Schedule => tab_schedule::draw(state, e, ui, out),
        DetailsTab::Relations => tab_relations::draw(state, e, ui, out),
        DetailsTab::Learning => tab_learning::draw(e, ui),
    }
}
