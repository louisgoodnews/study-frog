//! Geometry of the Home statistic ring and the headline gradient (EPIC-070, STORY-0214). Pure: points and meshes
//! only, so the drawing can be tested without a window.

use eframe::egui::{Color32, Mesh, Pos2, Rect};
use std::f32::consts::{FRAC_PI_2, TAU};

/// The part of a whole that `done` makes up, clamped to 0 to 1; zero when there is no total.
pub fn fraction(done: u32, total: u32) -> f32 {
    if total == 0 {
        0.0
    } else {
        (done as f32 / total as f32).clamp(0.0, 1.0)
    }
}

/// Points along an arc of `radius` around `centre`, starting at the top and running clockwise through `fraction` of a
/// full turn, with `segments` steps (at least one).
pub fn arc_points(centre: Pos2, radius: f32, fraction: f32, segments: usize) -> Vec<Pos2> {
    let steps = segments.max(1);
    let sweep = TAU * fraction.clamp(0.0, 1.0);
    (0..=steps)
        .map(|i| {
            let angle = -FRAC_PI_2 + sweep * (i as f32 / steps as f32);
            Pos2::new(
                centre.x + radius * angle.cos(),
                centre.y + radius * angle.sin(),
            )
        })
        .collect()
}

/// A rectangle filled with a left-to-right gradient from `from` to `to`.
pub fn gradient_mesh(rect: Rect, from: Color32, to: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), from);
    mesh.colored_vertex(rect.right_top(), to);
    mesh.colored_vertex(rect.right_bottom(), to);
    mesh.colored_vertex(rect.left_bottom(), from);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::palette;
    use eframe::egui::{Vec2, pos2};

    #[test]
    fn fraction_is_done_over_total_and_clamped() {
        assert_eq!(fraction(0, 0), 0.0);
        assert_eq!(fraction(3, 4), 0.75);
        assert_eq!(fraction(9, 4), 1.0);
    }

    #[test]
    fn a_full_arc_returns_to_the_top() {
        let c = pos2(0.0, 0.0);
        let pts = arc_points(c, 10.0, 1.0, 8);
        assert_eq!(pts.len(), 9);
        assert!((pts[0].x - 0.0).abs() < 1e-4 && (pts[0].y + 10.0).abs() < 1e-4);
        assert!((pts[8].x - 0.0).abs() < 1e-3 && (pts[8].y + 10.0).abs() < 1e-3);
    }

    #[test]
    fn a_half_arc_ends_at_the_bottom() {
        let pts = arc_points(pos2(0.0, 0.0), 10.0, 0.5, 4);
        let last = pts.last().copied().unwrap_or(pos2(0.0, 0.0));
        assert!(last.x.abs() < 1e-3 && (last.y - 10.0).abs() < 1e-3);
    }

    #[test]
    fn the_gradient_mesh_has_two_triangles_and_both_colours() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(10.0, 4.0));
        let (from, to) = (palette(false).background, palette(true).background);
        let mesh = gradient_mesh(rect, from, to);
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert_eq!(mesh.vertices[0].color, from);
        assert_eq!(mesh.vertices[1].color, to);
    }
}
