//! Stylized Breviceps macrops brand mascot icon (STORY-0146).

use eframe::egui::{self, Color32, Stroke, vec2};

/// The logo: a stylized Breviceps macrops (desert rain frog) with its globular body,
/// expressive large eyes with catchlights, and studious demeanor.
pub fn frog(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
    let c = rect.center();
    let p = ui.painter();

    let is_dark = ui.visuals().dark_mode;
    let skin_base = if is_dark {
        Color32::from_rgb(112, 196, 125)
    } else {
        Color32::from_rgb(88, 168, 102)
    };
    let belly_tint = if is_dark {
        Color32::from_rgb(148, 222, 160)
    } else {
        Color32::from_rgb(120, 194, 132)
    };
    let pupil_color = Color32::from_rgb(18, 32, 22);
    let eye_rim = if is_dark {
        Color32::from_rgb(140, 218, 150)
    } else {
        Color32::from_rgb(72, 145, 84)
    };
    let catchlight = Color32::WHITE;

    // Stubby little feet at the bottom
    p.circle_filled(egui::pos2(c.x - 7.5, c.y + 7.5), 2.2, skin_base);
    p.circle_filled(egui::pos2(c.x + 7.5, c.y + 7.5), 2.2, skin_base);

    // Globular, stout spherical body (signature Breviceps silhouette)
    p.add(egui::Shape::ellipse_filled(
        egui::pos2(c.x, c.y + 1.5),
        vec2(9.8, 8.2),
        skin_base,
    ));

    // Lighter chubby underbelly
    p.add(egui::Shape::ellipse_filled(
        egui::pos2(c.x, c.y + 4.2),
        vec2(6.5, 4.2),
        belly_tint,
    ));

    // Large, round, expressive forward-facing eyes
    let eye_y = c.y - 3.2;
    // Left eye base, pupil & catchlight
    p.circle_filled(egui::pos2(c.x - 5.0, eye_y), 4.2, eye_rim);
    p.circle_filled(egui::pos2(c.x - 5.0, eye_y), 3.4, pupil_color);
    p.circle_filled(egui::pos2(c.x - 6.0, eye_y - 1.2), 1.2, catchlight);
    p.circle_filled(egui::pos2(c.x - 4.2, eye_y + 1.0), 0.6, catchlight);

    // Right eye base, pupil & catchlight
    p.circle_filled(egui::pos2(c.x + 5.0, eye_y), 4.2, eye_rim);
    p.circle_filled(egui::pos2(c.x + 5.0, eye_y), 3.4, pupil_color);
    p.circle_filled(egui::pos2(c.x + 4.0, eye_y - 1.2), 1.2, catchlight);
    p.circle_filled(egui::pos2(c.x + 5.8, eye_y + 1.0), 0.6, catchlight);

    // Serious studious little mouth line (the iconic Breviceps expression)
    let mouth_y = c.y + 2.8;
    p.line_segment(
        [
            egui::pos2(c.x - 2.8, mouth_y + 0.5),
            egui::pos2(c.x, mouth_y),
        ],
        Stroke::new(1.0, pupil_color),
    );
    p.line_segment(
        [
            egui::pos2(c.x, mouth_y),
            egui::pos2(c.x + 2.8, mouth_y + 0.5),
        ],
        Stroke::new(1.0, pupil_color),
    );
}
