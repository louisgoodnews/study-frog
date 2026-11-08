//! The Leeches tab of the reports, and the same lists on a stack's Reports tab: items that keep failing with the
//! reasons, and pairs that seem to interfere, each with actions to open, rewrite, pause or drill (STORY-0104).

use eframe::egui;
use studyfrog_report::leech::LeechReason;
use studyfrog_service::analytics_leech::{LeechRow, PairRow};

use crate::explorer_ui::ExplorerMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit::{self, READING_WIDTH};

fn reason(r: LeechReason) -> &'static str {
    match r {
        LeechReason::RepeatedLapses => "forgotten again and again",
        LeechReason::StruggleStreak => "hard or forgotten several times in a row",
        LeechReason::HighLatencyBurden => "takes far longer than the stack's other items",
        LeechReason::StagnantStability => "many reviews, still a short memory",
    }
}

fn facts(r: &LeechRow) -> String {
    let l = &r.leech;
    let burden = l.time_burden.map_or(String::new(), |b| {
        format!(", {b:.1} times the stack's usual time")
    });
    let stability = l.efficiency.map_or(String::new(), |e| {
        format!(", {:.1} days of stability per review", e)
    });
    format!(
        "{} lapse(s), {} review(s), {} hard or forgotten in a row{burden}{stability}",
        l.lapses, l.reviews, l.streak
    )
}

fn leech_row(r: &LeechRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            if ui.link(&r.title).clicked() {
                out.push(Msg::OpenItem(r.uid));
            }
            let why: Vec<&str> = r.leech.reasons.iter().map(|x| reason(*x)).collect();
            ui.label(format!("- {}", why.join("; ")));
        });
        ui.weak(facts(r));
        ui.horizontal_wrapped(|ui| {
            if ui.button(format!("Edit or split {}", r.title)).clicked() {
                out.push(Msg::OpenItem(r.uid));
            }
            if ui.button(format!("Pause {}", r.title)).clicked() {
                out.push(Msg::Explorer(ExplorerMsg::Pause(r.uid)));
            }
            if ui.button(format!("Drill {}", r.title)).clicked() {
                out.push(Msg::Explorer(ExplorerMsg::Drill(vec![r.uid])));
            }
        });
    });
}

fn pair_row(p: &PairRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let x = &p.pair;
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            if ui.link(&p.a.1).clicked() {
                out.push(Msg::OpenItem(p.a.0));
            }
            ui.label("and");
            if ui.link(&p.b.1).clicked() {
                out.push(Msg::OpenItem(p.b.0));
            }
        });
        let mut why = vec![format!(
            "forgotten within a day of each other {} time(s)",
            x.co_lapses
        )];
        why.push(format!("{:.0} % shared words", x.similarity * 100.0));
        if x.shared_tags > 0 {
            why.push(format!("{} shared tag(s)", x.shared_tags));
        }
        if x.linked {
            why.push("linked".to_owned());
        }
        ui.weak(why.join(", "));
        if ui
            .button(format!("Drill {} and {} together", p.a.1, p.b.1))
            .clicked()
        {
            out.push(Msg::Explorer(ExplorerMsg::Drill(vec![p.a.0, p.b.0])));
        }
    });
}

/// The two lists, for whatever rows the caller chose.
pub fn lists(leeches: &[&LeechRow], pairs: &[&PairRow], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Leeches");
    ui.weak("Items that keep failing or eat time: at least 4 lapses, 3 hard or forgotten in a row, 3 times the stack's usual time, or 6 reviews and still under 7 days of stability.");
    if leeches.is_empty() {
        ui.label("No leeches.");
    }
    for r in leeches {
        leech_row(r, ui, out);
    }
    kit::section(ui, "Items that get confused");
    ui.weak("Pairs forgotten close together that look alike, share a tag or link to each other. Drill them side by side to tell them apart.");
    if pairs.is_empty() {
        ui.label("No pairs that get confused.");
    }
    for p in pairs {
        pair_row(p, ui, out);
    }
}

/// The tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(r) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let leeches: Vec<&LeechRow> = r.analytics.leeches.iter().collect();
    let pairs: Vec<&PairRow> = r.analytics.pairs.iter().collect();
    egui::ScrollArea::vertical().show(ui, |ui| {
        kit::column(ui, READING_WIDTH, |ui| lists(&leeches, &pairs, ui, out));
    });
}

/// Whether `folder` is the stack at `stack` or inside it.
pub fn in_stack(folder: &str, stack: &str) -> bool {
    folder == stack || folder.starts_with(&format!("{stack}/"))
}
