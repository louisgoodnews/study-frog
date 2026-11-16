//! Viewport culling with spatial quadtree and canvas element painters.
//!
//! Follows Invariant P1: zero inherent impl blocks; logic in free functions.

use std::collections::HashSet;

use eframe::egui::{Color32, FontId, Painter, Pos2, Rect as EguiRect, Stroke, Vec2};
use studyfrog_graph::canvas::{CanvasFrame, CanvasNode, CanvasNodeKind};
use studyfrog_graph::quadtree::{
    Quadtree, Rect as QuadRect, insert, new_quadtree, new_rect, new_spatial_item, query_range,
};

use crate::canvas_types::{CanvasTransform, world_to_screen};
use crate::theme::{Palette, tint};

/// Constructs a spatial quadtree index from the given canvas nodes.
pub fn build_node_quadtree(nodes: &[CanvasNode]) -> Quadtree<String> {
    if nodes.is_empty() {
        return new_quadtree(new_rect(-10_000.0, -10_000.0, 20_000.0, 20_000.0), 8, 6);
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for n in nodes {
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + n.width);
        max_y = max_y.max(n.y + n.height);
    }

    let pad = 1_000.0;
    let bounds = new_rect(
        min_x - pad,
        min_y - pad,
        (max_x - min_x + pad * 2.0).max(100.0),
        (max_y - min_y + pad * 2.0).max(100.0),
    );

    let mut tree = new_quadtree(bounds, 8, 6);
    for n in nodes {
        let item_rect = new_rect(n.x, n.y, n.width.max(1.0), n.height.max(1.0));
        let item = new_spatial_item(&n.id, item_rect, n.id.clone());
        insert(&mut tree, item);
    }
    tree
}

/// Returns nodes whose bounding boxes intersect the visible viewport range.
pub fn cull_visible_nodes<'a>(
    tree: &Quadtree<String>,
    nodes: &'a [CanvasNode],
    view_rect: &QuadRect,
) -> Vec<&'a CanvasNode> {
    if view_rect.w <= 0.0 || view_rect.h <= 0.0 {
        return Vec::new();
    }
    let mut hits = Vec::new();
    query_range(tree, view_rect, &mut hits);
    let visible_ids: HashSet<&str> = hits.iter().map(|it| it.id.as_str()).collect();
    nodes
        .iter()
        .filter(|n| visible_ids.contains(n.id.as_str()))
        .collect()
}

/// Paints subtle background grid dots corresponding to canvas world coordinates.
pub fn paint_grid(painter: &Painter, tx: &CanvasTransform, screen_rect: EguiRect, color: Color32) {
    let world_step = 60.0;
    let screen_step = (world_step * tx.zoom) as f32;
    if screen_step < 12.0 {
        return;
    }

    let center = (screen_rect.center().x, screen_rect.center().y);
    let (min_wx, min_wy) =
        crate::canvas_types::screen_to_world(tx, screen_rect.min.x, screen_rect.min.y, center);
    let (max_wx, max_wy) =
        crate::canvas_types::screen_to_world(tx, screen_rect.max.x, screen_rect.max.y, center);

    let start_x = (min_wx / world_step).floor() * world_step;
    let end_x = (max_wx / world_step).ceil() * world_step;
    let start_y = (min_wy / world_step).floor() * world_step;
    let end_y = (max_wy / world_step).ceil() * world_step;

    let dot_radius = (1.2 * tx.zoom as f32).clamp(1.0, 2.5);
    let mut wx = start_x;
    while wx <= end_x {
        let mut wy = start_y;
        while wy <= end_y {
            let (sx, sy) = world_to_screen(tx, wx, wy, center);
            painter.circle_filled(Pos2::new(sx, sy), dot_radius, color);
            wy += world_step;
        }
        wx += world_step;
    }
}

/// Paints a grouping frame boundary and header.
pub fn paint_frame(
    painter: &Painter,
    frame: &CanvasFrame,
    tx: &CanvasTransform,
    center: (f32, f32),
    p: &Palette,
) {
    let (sx, sy) = world_to_screen(tx, frame.x, frame.y, center);
    let sw = (frame.width * tx.zoom) as f32;
    let sh = (frame.height * tx.zoom) as f32;
    let rect = EguiRect::from_min_size(Pos2::new(sx, sy), Vec2::new(sw, sh));

    painter.rect_filled(rect, 8.0, tint(p.panel, 40));
    painter.rect_stroke(
        rect,
        8.0,
        Stroke::new(1.5, tint(p.alt, 100)),
        eframe::egui::StrokeKind::Inside,
    );

    if !frame.title.is_empty() && sw > 50.0 {
        let font_id = crate::typo::painter_font(painter, crate::typo::Tier::Caption);
        let tag = painter.layout_no_wrap(frame.title.clone(), font_id, p.text);
        let tag_rect = EguiRect::from_min_size(
            Pos2::new(sx + 8.0, sy + 6.0),
            tag.size() + Vec2::new(8.0, 4.0),
        );
        painter.rect_filled(tag_rect, 4.0, tint(p.surface, 180));
        painter.galley(Pos2::new(sx + 12.0, sy + 8.0), tag, p.text);
    }
}

/// Paints a single canvas node card with selection and hover styling.
pub fn paint_node(
    painter: &Painter,
    node: &CanvasNode,
    tx: &CanvasTransform,
    center: (f32, f32),
    (selected, hovered): (bool, bool),
    p: &Palette,
) {
    let (sx, sy) = world_to_screen(tx, node.x, node.y, center);
    let sw = (node.width * tx.zoom) as f32;
    let sh = (node.height * tx.zoom) as f32;
    let rect = EguiRect::from_min_size(Pos2::new(sx, sy), Vec2::new(sw, sh));

    let stroke = if selected {
        Stroke::new(2.5, p.accent)
    } else if hovered {
        Stroke::new(1.8, tint(p.accent, 160))
    } else {
        Stroke::new(1.0, p.border)
    };

    let bg = if hovered { p.surface_hover } else { p.surface };
    painter.rect_filled(rect, 6.0, bg);
    painter.rect_stroke(rect, 6.0, stroke, eframe::egui::StrokeKind::Inside);

    if sw < 30.0 || sh < 15.0 {
        return;
    }

    let kind_label = match node.kind {
        CanvasNodeKind::Item => "CARD",
        CanvasNodeKind::Card => "CARD",
        CanvasNodeKind::Note => "NOTE",
        CanvasNodeKind::Text => "TEXT",
    };
    let kind_color = match node.kind {
        CanvasNodeKind::Item | CanvasNodeKind::Card => p.success,
        CanvasNodeKind::Note => p.alt,
        CanvasNodeKind::Text => p.text_weak,
    };

    let title_text = node
        .label
        .as_deref()
        .unwrap_or_else(|| node.item_id.as_deref().unwrap_or(&node.id));

    let badge_font = FontId::proportional((9.0 * tx.zoom as f32).clamp(8.0, 11.0));
    let badge_galley = painter.layout_no_wrap(kind_label.to_string(), badge_font, kind_color);
    painter.galley(Pos2::new(sx + 8.0, sy + 6.0), badge_galley, kind_color);

    if sh > 35.0 {
        let title_font = FontId::proportional((12.0 * tx.zoom as f32).clamp(10.0, 14.0));
        let title_galley = painter.layout(
            title_text.to_string(),
            title_font,
            p.text,
            (sw - 16.0).max(10.0),
        );
        painter.galley(Pos2::new(sx + 8.0, sy + 22.0), title_galley, p.text);
    }
}
