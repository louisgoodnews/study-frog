//! The Relations tab of the item details notebook: primary stack info, outbound wikilinks,
//! and inbound backlinks.

use eframe::egui;
use studyfrog_md::blocks::{Block, Link, Span};
use studyfrog_md::doc::Doc;

use crate::editor::Editor;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

/// Draw the Relations tab.
pub fn draw(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !matches!(e.doc, Doc::Stack(_)) {
        crate::view::item_stacks::card(state, e, ui, out);
        ui.add_space(10.0);
    }
    links(e, ui, out);
    ui.add_space(10.0);
    linked_from(e, ui, out);
}

fn wiki_targets(blocks: &[Block], into: &mut Vec<String>) {
    let spans = |spans: &[Span], into: &mut Vec<String>| {
        for s in spans {
            match &s.link {
                Some(Link::Wiki(t)) if !into.contains(t) => into.push(t.clone()),
                _ => {}
            }
        }
    };
    for b in blocks {
        match b {
            Block::Heading { spans: s, .. } | Block::Paragraph(s) => spans(s, into),
            Block::Quote(inner) => wiki_targets(inner, into),
            Block::List { items, .. } => items.iter().for_each(|i| wiki_targets(&i.blocks, into)),
            Block::Table { header, rows, .. } => {
                for cell in header.iter().chain(rows.iter().flatten()) {
                    spans(cell, into);
                }
            }
            Block::Code { .. } | Block::Rule | Block::Toc => {}
        }
    }
}

fn linked_from(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Linked from");
    super::item_card(e, ui, |ui| {
        ui.set_width(ui.available_width());
        if e.backlinks.is_empty() {
            ui.weak("Nothing links here yet.");
        }
        let weak = palette_of(ui).text_weak;
        for item in &e.backlinks {
            ui.horizontal(|ui| {
                kit::pill(ui, studyfrog_core::plan::item_kind_name(item.kind), weak);
                if ui.link(&item.title).clicked() {
                    out.push(Msg::OpenItem(item.uid));
                }
            });
        }
    });
}

fn links(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut targets = Vec::new();
    for f in &e.form.fields {
        wiki_targets(&studyfrog_md::blocks::parse_blocks(&f.text), &mut targets);
    }
    kit::section(ui, "Links to");
    super::item_card(e, ui, |ui| {
        ui.set_width(ui.available_width());
        if targets.is_empty() {
            ui.weak("No links yet. Write [[Title]] to link a note.");
        }
        for t in targets {
            if ui.link(&t).clicked() {
                out.push(Msg::OpenLink(t));
            }
        }
    });
}
