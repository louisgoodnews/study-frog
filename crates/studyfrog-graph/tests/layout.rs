#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Correctness of the layout: the grid against a brute-force oracle, convergence, bounds.

use proptest::prelude::*;
use studyfrog_graph::layout::{Layout, layout_new, layout_step};
use studyfrog_graph::params::{Params, params_default};
use studyfrog_graph::view::{View, nearest, project, view_fit};

fn run(n: usize, edges: &[(u32, u32)], steps: usize, seed: u64, p: &Params) -> Layout {
    let mut l = layout_new(n, seed, p);
    for _ in 0..steps {
        layout_step(&mut l, edges, p);
    }
    l
}

fn dist(l: &Layout, a: usize, b: usize) -> f32 {
    ((l.x[a] - l.x[b]).powi(2) + (l.y[a] - l.y[b]).powi(2)).sqrt()
}

#[test]
fn equal_seeds_give_equal_layouts_and_different_seeds_do_not() {
    let p = params_default();
    let edges: Vec<_> = (0..30).map(|i| (i, (i + 1) % 30)).collect();
    assert_eq!(run(30, &edges, 40, 7, &p), run(30, &edges, 40, 7, &p));
    assert_ne!(run(30, &edges, 40, 7, &p).x, run(30, &edges, 40, 8, &p).x);
}

#[test]
fn the_grid_finds_exactly_the_repulsion_a_brute_force_pass_finds() {
    let p = Params {
        spring: 0.0,
        gravity: 0.0,
        damping: 1.0,
        max_step: 1e9,
        ..params_default()
    };
    let mut l = layout_new(120, 3, &p);
    // Spread nodes over several cells so both near and far pairs exist.
    for i in 0..120 {
        let t = i as f32;
        l.x[i] = ((t * 37.0) % 400.0) - 200.0;
        l.y[i] = ((t * 91.0) % 400.0) - 200.0;
    }
    let (px, py) = (l.x.clone(), l.y.clone());
    layout_step(&mut l, &[], &p);
    for i in 0..120 {
        let (mut fx, mut fy) = (0.0_f32, 0.0_f32);
        for j in (0..120).filter(|j| *j != i) {
            let (dx, dy) = (px[i] - px[j], py[i] - py[j]);
            let d2 = dx * dx + dy * dy;
            if d2 < p.cutoff * p.cutoff {
                let w = 1.0 - d2 / (p.cutoff * p.cutoff);
                let scale = p.repulsion / (d2 + 1.0) * w * w;
                fx += dx * scale;
                fy += dy * scale;
            }
        }
        // Starting from rest with damping 1, the velocity is exactly the force.
        assert!(
            (l.vx[i] - fx).abs() <= 1e-3 * (1.0 + fx.abs()),
            "node {i}: {} vs {fx}",
            l.vx[i]
        );
        assert!(
            (l.vy[i] - fy).abs() <= 1e-3 * (1.0 + fy.abs()),
            "node {i}: {} vs {fy}",
            l.vy[i]
        );
    }
}

#[test]
fn springs_settle_an_edge_near_its_rest_length() {
    let p = Params {
        repulsion: 0.0,
        gravity: 0.0,
        ..params_default()
    };
    let l = run(2, &[(0, 1)], 600, 1, &p);
    assert!(
        (dist(&l, 0, 1) - p.rest_length).abs() < 1.0,
        "{}",
        dist(&l, 0, 1)
    );
}

#[test]
fn connected_groups_end_up_closer_than_unconnected_ones() {
    let p = params_default();
    let tri = |o: u32| [(o, o + 1), (o + 1, o + 2), (o + 2, o)];
    let edges: Vec<_> = tri(0).into_iter().chain(tri(3)).collect();
    let l = run(6, &edges, 800, 5, &p);
    let within = (dist(&l, 0, 1) + dist(&l, 1, 2) + dist(&l, 3, 4) + dist(&l, 4, 5)) / 4.0;
    let between = (dist(&l, 0, 3) + dist(&l, 1, 4) + dist(&l, 2, 5)) / 3.0;
    assert!(within < between, "within {within} between {between}");
}

#[test]
fn every_kind_of_graph_comes_to_rest() {
    let p = params_default();
    let ring: Vec<_> = (0..100).map(|i| (i, (i + 1) % 100)).collect();
    let star: Vec<_> = (1..40).map(|i| (0, i)).collect();
    for (name, n, edges) in [
        ("ring", 100, ring),
        ("star", 40, star),
        ("isolated", 200, vec![]),
    ] {
        let mut l = layout_new(n, 2, &p);
        let early = (0..30)
            .map(|_| layout_step(&mut l, &edges, &p))
            .last()
            .unwrap();
        let late = (0..1500)
            .map(|_| layout_step(&mut l, &edges, &p))
            .last()
            .unwrap();
        assert!(
            late / (n as f32) < 0.01,
            "{name}: per-node energy {} -> {}",
            early / n as f32,
            late / n as f32
        );
    }
}

#[test]
fn degenerate_layouts_do_not_panic() {
    let p = params_default();
    for n in [0, 1, 2] {
        let l = run(n, &[], 5, 1, &p);
        assert_eq!(l.x.len(), n);
    }
}

#[test]
fn projection_and_picking() {
    let p = params_default();
    let l = run(3, &[(0, 1)], 20, 1, &p);
    let v = View {
        zoom: 2.0,
        center_x: l.x[0],
        center_y: l.y[0],
    };
    let (mut sx, mut sy) = (vec![0.0; 3], vec![0.0; 3]);
    project(&l, &v, 100.0, 50.0, &mut sx, &mut sy);
    assert_eq!(
        (sx[0], sy[0]),
        (100.0, 50.0),
        "the centred node lands on the origin"
    );
    assert_eq!(nearest(&sx, &sy, 101.0, 51.0, 5.0), Some(0));
    assert_eq!(nearest(&sx, &sy, -9999.0, 0.0, 5.0), None);
    assert!(view_fit(&l, 800.0, 600.0).zoom > 0.0);
}

proptest! {
    #[test]
    fn positions_stay_finite_and_inside_the_square(
        n in 0_usize..60,
        edges in prop::collection::vec((0_u32..60, 0_u32..60), 0..120),
        seed in any::<u64>(),
    ) {
        let p = params_default();
        let edges: Vec<_> = edges.into_iter().filter(|(a, b)| (*a as usize) < n && (*b as usize) < n).collect();
        let l = run(n, &edges, 80, seed, &p);
        for i in 0..n {
            prop_assert!(l.x[i].is_finite() && l.y[i].is_finite());
            prop_assert!(l.x[i].abs() <= l.extent && l.y[i].abs() <= l.extent);
        }
    }
}
