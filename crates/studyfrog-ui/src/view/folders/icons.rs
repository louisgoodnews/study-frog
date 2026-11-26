//! Vector-drawn Nautilus/Thunar-style icons for the Folders view (EPIC-074).

use eframe::egui::{self, Align2, CornerRadius, FontId, Pos2, Rect, Stroke, Vec2, pos2};

use crate::folders_types::{EntryKind, FolderEntry};
use crate::theme::{Palette, palette_of};

/// Paint a recognizable folder or file icon scaled to `size`.
pub fn paint_entry_icon(ui: &mut egui::Ui, entry: &FolderEntry, size: Vec2) -> Rect {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let pal = palette_of(ui);
    let painter = ui.painter();

    if entry.is_folder {
        paint_folder(painter, rect, entry.is_stack, &pal);
    } else {
        paint_file(painter, rect, &entry.kind, &pal);
    }
    rect
}

fn paint_folder(painter: &egui::Painter, rect: Rect, is_stack: bool, pal: &Palette) {
    let fill = if is_stack {
        pal.accent
    } else {
        pal.surface_hover
    };
    let tab_fill = if is_stack {
        pal.accent.gamma_multiply(0.85)
    } else {
        pal.border
    };
    let w = rect.width();
    let h = rect.height();

    // Folder back tab
    let tab = Rect::from_min_max(
        pos2(rect.left() + w * 0.06, rect.top() + h * 0.12),
        pos2(rect.left() + w * 0.48, rect.top() + h * 0.36),
    );
    painter.rect_filled(
        tab,
        CornerRadius::same((w * 0.08).clamp(2.0, 6.0) as u8),
        tab_fill,
    );

    // Folder main body
    let body = Rect::from_min_max(
        pos2(rect.left() + w * 0.06, rect.top() + h * 0.24),
        pos2(rect.right() - w * 0.06, rect.bottom() - h * 0.10),
    );
    let r = (w * 0.10).clamp(2.0, 8.0) as u8;
    painter.rect_filled(body, CornerRadius::same(r), fill);
    painter.rect_stroke(
        body,
        CornerRadius::same(r),
        Stroke::new(1.0, pal.border),
        egui::StrokeKind::Inside,
    );

    if is_stack && w >= 28.0 {
        painter.text(
            body.center(),
            Align2::CENTER_CENTER,
            "S",
            FontId::proportional((h * 0.32).clamp(9.0, 18.0)),
            pal.text,
        );
    }
}

fn paint_file(painter: &egui::Painter, rect: Rect, kind: &EntryKind, pal: &Palette) {
    let w = rect.width();
    let h = rect.height();
    let page = Rect::from_min_max(
        pos2(rect.left() + w * 0.16, rect.top() + h * 0.08),
        pos2(rect.right() - w * 0.16, rect.bottom() - h * 0.08),
    );
    let r = (w * 0.08).clamp(2.0, 6.0) as u8;
    painter.rect_filled(page, CornerRadius::same(r), pal.surface_hover);
    painter.rect_stroke(
        page,
        CornerRadius::same(r),
        Stroke::new(1.0, pal.border),
        egui::StrokeKind::Inside,
    );

    // Folded top-right corner accent
    let fold = (page.width() * 0.28).max(4.0);
    let corner = vec![
        Pos2::new(page.right() - fold, page.top()),
        Pos2::new(page.right(), page.top() + fold),
        Pos2::new(page.right() - fold, page.top() + fold),
    ];
    painter.add(egui::Shape::convex_polygon(
        corner,
        pal.border,
        Stroke::NONE,
    ));

    match kind {
        EntryKind::Markdown => {
            for idx in 0..3 {
                let y = page.top() + page.height() * (0.42 + idx as f32 * 0.16);
                let x2 = if idx == 2 {
                    page.right() - page.width() * 0.30
                } else {
                    page.right() - page.width() * 0.18
                };
                painter.line_segment(
                    [pos2(page.left() + page.width() * 0.18, y), pos2(x2, y)],
                    Stroke::new(1.5, pal.text_weak),
                );
            }
        }
        EntryKind::Pdf => {
            let band = Rect::from_min_max(
                pos2(page.left() + 2.0, page.center().y - page.height() * 0.10),
                pos2(page.right() - 2.0, page.center().y + page.height() * 0.22),
            );
            painter.rect_filled(band, CornerRadius::same(2), pal.accent);
            if w >= 28.0 {
                painter.text(
                    band.center(),
                    Align2::CENTER_CENTER,
                    "PDF",
                    FontId::proportional((h * 0.20).clamp(8.0, 12.0)),
                    pal.text,
                );
            }
        }
        EntryKind::Image => {
            let inner = page.shrink2(Vec2::new(page.width() * 0.16, page.height() * 0.22));
            painter.circle_filled(
                pos2(
                    inner.left() + inner.width() * 0.28,
                    inner.top() + inner.height() * 0.28,
                ),
                (inner.width() * 0.14).max(2.0),
                pal.warning,
            );
            let hill = vec![
                pos2(inner.left(), inner.bottom()),
                pos2(inner.center().x, inner.top() + inner.height() * 0.35),
                pos2(inner.right(), inner.bottom()),
            ];
            painter.add(egui::Shape::convex_polygon(hill, pal.accent, Stroke::NONE));
        }
        EntryKind::Other(ext) => {
            if w >= 28.0 {
                let tag: String = ext.chars().take(4).collect();
                painter.text(
                    page.center(),
                    Align2::CENTER_CENTER,
                    tag,
                    FontId::proportional((h * 0.20).clamp(8.0, 12.0)),
                    pal.text_weak,
                );
            }
        }
        EntryKind::Folder => {}
    }
}
