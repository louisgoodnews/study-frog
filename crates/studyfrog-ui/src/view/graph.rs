//! The link graph canvas: nodes and edges drawn from the screen positions computed in `update`.

use eframe::egui::{self, Color32, Pos2, Sense, Stroke};
use studyfrog_core::plan::ItemKind;
use studyfrog_graph::view::nearest;

use crate::graph::GraphState;
use crate::msg::Msg;
use crate::state::State;

const NODE_RADIUS: f32 = 3.5;
const PICK_RADIUS: f32 = 9.0;
/// Titles are drawn next to nodes only when zoomed in this far (and for the hovered node).
const LABEL_ZOOM: f32 = 3.0;
const MAX_LABELS: usize = 150;
/// How many items the accessible list offers.
const LIST_LIMIT: usize = 200;

fn kind_color(p: &crate::theme::Palette, kind: ItemKind) -> Color32 {
    match kind {
        ItemKind::Note => p.alt,
        ItemKind::Card => p.success,
        ItemKind::Question => p.warning,
    }
}

/// Header, canvas and the accessible item list.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(
        egui::RichText::new("Link graph")
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
    );
    ui.add_space(4.0);
    let Some(g) = &state.graph else {
        ui.label("Loading the graph…");
        return;
    };
    let (n, m) = (g.graph.nodes.len(), g.graph.edges.len());
    ui.horizontal(|ui| {
        ui.label(format!("{n} items, {m} links"));
        if ui.button("Re-run layout").clicked() {
            out.push(Msg::GraphRestart);
        }
        ui.weak("Drag to pan, scroll to zoom, click an item to open it.");
    });
    if n == 0 {
        ui.label("There are no items yet.");
        return;
    }
    canvas(g, ui, out);
    ui.add_space(6.0);
    if let Some(i) = g.hovered {
        ui.label(format!("Pointing at: {}", g.graph.nodes[i].title));
    }
    item_list(g, ui, out);
}

fn canvas(g: &GraphState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let height = (ui.available_height() - 90.0).max(200.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        Sense::click_and_drag(),
    );
    out.push(Msg::GraphFrame {
        width: rect.width(),
        height: rect.height(),
    });
    if response.dragged() {
        let d = response.drag_delta();
        out.push(Msg::GraphPan(d.x, d.y));
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            out.push(Msg::GraphZoom((scroll * 0.002).exp()));
        }
    }
    let center = rect.center();
    let hover = response
        .hover_pos()
        .and_then(|p| nearest(&g.sx, &g.sy, p.x - center.x, p.y - center.y, PICK_RADIUS));
    if hover != g.hovered {
        out.push(Msg::GraphHover(hover));
    }
    if response.clicked()
        && let Some(i) = hover
    {
        out.push(Msg::GraphClick(i));
    }
    paint(g, ui, rect);
    if !g.settled {
        ui.ctx().request_repaint();
    }
}

fn paint(g: &GraphState, ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter_at(rect);
    let pal = crate::theme::palette_of(ui);
    painter.rect(
        rect,
        8.0,
        ui.visuals().extreme_bg_color,
        Stroke::new(1.0, pal.border),
        egui::StrokeKind::Inside,
    );
    let c = rect.center();
    let at = |i: usize| Pos2::new(c.x + g.sx[i], c.y + g.sy[i]);
    let visible = rect.expand(40.0);
    let edge = Stroke::new(1.0, ui.visuals().weak_text_color().gamma_multiply(0.5));
    for &(a, b) in &g.graph.edges {
        let (pa, pb) = (at(a as usize), at(b as usize));
        if visible.contains(pa) || visible.contains(pb) {
            painter.line_segment([pa, pb], edge);
        }
    }
    let mut labels = 0;
    for (i, node) in g.graph.nodes.iter().enumerate() {
        let p = at(i);
        if !visible.contains(p) {
            continue;
        }
        let hovered = g.hovered == Some(i);
        painter.circle_filled(
            p,
            if hovered {
                NODE_RADIUS * 1.8
            } else {
                NODE_RADIUS
            },
            kind_color(&pal, node.kind),
        );
        if hovered || (g.view.zoom >= LABEL_ZOOM && labels < MAX_LABELS) {
            labels += 1;
            painter.text(
                p + egui::vec2(6.0, -6.0),
                egui::Align2::LEFT_BOTTOM,
                &node.title,
                crate::typo::font(ui, crate::typo::Tier::Caption),
                ui.visuals().text_color(),
            );
        }
    }
}

/// The canvas cannot be read by a screen reader, so the same items are offered as links.
fn item_list(g: &GraphState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.collapsing("Items as a list", |ui| {
        egui::ScrollArea::vertical()
            .max_height(160.0)
            .show(ui, |ui| {
                for node in g.graph.nodes.iter().take(LIST_LIMIT) {
                    if ui.link(&node.title).clicked() {
                        out.push(Msg::OpenItem(node.uid));
                    }
                }
                if g.graph.nodes.len() > LIST_LIMIT {
                    ui.weak(format!(
                        "… and {} more (use the graph or the stack browser)",
                        g.graph.nodes.len() - LIST_LIMIT
                    ));
                }
            });
    });
}
