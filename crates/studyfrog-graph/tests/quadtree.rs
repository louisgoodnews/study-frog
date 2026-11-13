#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for spatial quadtree indexing and viewport culling.

use std::time::Instant;
use studyfrog_graph::quadtree::{
    clear, count, insert, new_quadtree, new_rect, new_spatial_item, query_point, query_range,
    rect_contains_point, rect_contains_rect, rect_intersects,
};

#[test]
fn rect_geometry_operations() {
    let r1 = new_rect(10.0, 10.0, 20.0, 20.0);
    let r2 = new_rect(25.0, 25.0, 20.0, 20.0);
    let r3 = new_rect(30.0, 10.0, 10.0, 20.0); // touches r1 at x=30
    let r_far = new_rect(100.0, 100.0, 10.0, 10.0);
    let r_neg = new_rect(10.0, 10.0, -5.0, 20.0);

    // Intersections
    assert!(rect_intersects(&r1, &r2));
    assert!(rect_intersects(&r1, &r3)); // touching edge
    assert!(!rect_intersects(&r1, &r_far));
    assert!(!rect_intersects(&r1, &r_neg));

    // Point containment
    assert!(rect_contains_point(&r1, 10.0, 10.0)); // top-left corner
    assert!(rect_contains_point(&r1, 30.0, 30.0)); // bottom-right corner
    assert!(rect_contains_point(&r1, 20.0, 20.0)); // center
    assert!(!rect_contains_point(&r1, 9.9, 20.0));
    assert!(!rect_contains_point(&r1, 20.0, 30.1));
    assert!(!rect_contains_point(&r_neg, 10.0, 10.0));

    // Rect containment
    let r_inner = new_rect(12.0, 12.0, 5.0, 5.0);
    assert!(rect_contains_rect(&r1, &r1));
    assert!(rect_contains_rect(&r1, &r_inner));
    assert!(!rect_contains_rect(&r_inner, &r1));
    assert!(!rect_contains_rect(&r1, &r2));
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn quadtree_insert_subdivide_and_clear() {
    let bounds = new_rect(0.0, 0.0, 100.0, 100.0);
    let mut tree = new_quadtree::<i32>(bounds, 2, 3);
    assert_eq!(count(&tree), 0);
    assert!(tree.children.is_none());

    // Out of bounds insertion fails
    let out = new_spatial_item("out", new_rect(95.0, 95.0, 10.0, 10.0), 1);
    assert!(!insert(&mut tree, out));

    // Insert 2 items in NW (0..50, 0..50)
    assert!(insert(
        &mut tree,
        new_spatial_item("nw1", new_rect(5.0, 5.0, 10.0, 10.0), 10)
    ));
    assert!(insert(
        &mut tree,
        new_spatial_item("nw2", new_rect(20.0, 20.0, 5.0, 5.0), 20)
    ));
    assert_eq!(count(&tree), 2);
    assert!(tree.children.is_none());

    // 3rd item in SE triggers subdivision (capacity=2)
    assert!(insert(
        &mut tree,
        new_spatial_item("se1", new_rect(60.0, 60.0, 10.0, 10.0), 30)
    ));
    assert_eq!(count(&tree), 3);
    assert!(tree.children.is_some());

    // Item straddling the center stays at root
    assert!(insert(
        &mut tree,
        new_spatial_item("mid", new_rect(45.0, 45.0, 10.0, 10.0), 40)
    ));
    assert_eq!(count(&tree), 4);
    assert_eq!(tree.items.len(), 1);
    assert_eq!(tree.items[0].id, "mid");

    // Clear resets quadtree completely
    clear(&mut tree);
    assert_eq!(count(&tree), 0);
    assert!(tree.items.is_empty());
    assert!(tree.children.is_none());
}

#[test]
fn quadtree_max_depth_prevents_infinite_split() {
    let bounds = new_rect(0.0, 0.0, 100.0, 100.0);
    let mut tree = new_quadtree::<()>(bounds, 1, 0); // max_depth 0

    assert!(insert(
        &mut tree,
        new_spatial_item("a", new_rect(10.0, 10.0, 5.0, 5.0), ())
    ));
    assert!(insert(
        &mut tree,
        new_spatial_item("b", new_rect(20.0, 20.0, 5.0, 5.0), ())
    ));
    assert_eq!(count(&tree), 2);
    assert!(tree.children.is_none());
}

#[test]
fn query_range_and_point() {
    let bounds = new_rect(0.0, 0.0, 200.0, 200.0);
    let mut tree = new_quadtree::<&'static str>(bounds, 2, 4);

    let items = [
        new_spatial_item("card-1", new_rect(10.0, 10.0, 20.0, 20.0), "NW card"),
        new_spatial_item("card-2", new_rect(110.0, 10.0, 20.0, 20.0), "NE card"),
        new_spatial_item("card-3", new_rect(10.0, 110.0, 20.0, 20.0), "SW card"),
        new_spatial_item("card-4", new_rect(110.0, 110.0, 20.0, 20.0), "SE card"),
        new_spatial_item("frame", new_rect(5.0, 5.0, 190.0, 190.0), "Global frame"),
    ];

    for it in items {
        assert!(insert(&mut tree, it));
    }
    assert_eq!(count(&tree), 5);

    // Viewport query in top-left
    let vp_nw = new_rect(0.0, 0.0, 50.0, 50.0);
    let mut hits = Vec::new();
    query_range(&tree, &vp_nw, &mut hits);
    let mut hit_ids: Vec<_> = hits.iter().map(|it| it.id.as_str()).collect();
    hit_ids.sort();
    assert_eq!(hit_ids, vec!["card-1", "frame"]);

    // Viewport outside any item
    hits.clear();
    let vp_empty = new_rect(70.0, 70.0, 10.0, 10.0);
    query_range(&tree, &vp_empty, &mut hits);
    hit_ids = hits.iter().map(|it| it.id.as_str()).collect();
    assert_eq!(hit_ids, vec!["frame"]);

    // Point hit test
    hits.clear();
    query_point(&tree, 15.0, 15.0, &mut hits);
    hit_ids = hits.iter().map(|it| it.id.as_str()).collect();
    hit_ids.sort();
    assert_eq!(hit_ids, vec!["card-1", "frame"]);

    hits.clear();
    query_point(&tree, 199.0, 199.0, &mut hits);
    assert!(hits.is_empty());
}

#[test]
fn performance_1000_items_sub_millisecond_viewport_cull() {
    let bounds = new_rect(0.0, 0.0, 10_000.0, 10_000.0);
    let mut tree = new_quadtree::<usize>(bounds, 8, 6);

    let mut linear_items = Vec::with_capacity(1200);
    for i in 0..1200 {
        let x = (i * 37 % 9900) as f64;
        let y = (i * 73 % 9900) as f64;
        let item = new_spatial_item(format!("node-{i}"), new_rect(x, y, 50.0, 50.0), i);
        linear_items.push(item.clone());
        assert!(insert(&mut tree, item));
    }

    assert_eq!(count(&tree), 1200);

    // Viewport rectangle representing 1080p screen in canvas space
    let viewport = new_rect(2000.0, 2000.0, 1920.0, 1080.0);

    // Brute force ground truth
    let mut ground_truth: Vec<_> = linear_items
        .iter()
        .filter(|it| rect_intersects(&it.rect, &viewport))
        .map(|it| it.id.as_str())
        .collect();
    ground_truth.sort();

    // Quadtree query timing
    let mut hits = Vec::with_capacity(64);
    let start = Instant::now();
    for _ in 0..50 {
        hits.clear();
        query_range(&tree, &viewport, &mut hits);
    }
    let elapsed = start.elapsed();
    let per_query_us = (elapsed.as_micros() as f64) / 50.0;

    let mut hit_ids: Vec<_> = hits.iter().map(|it| it.id.as_str()).collect();
    hit_ids.sort();

    // Sub-millisecond requirement: per-query must be well under 1000 us (1 ms)
    assert_eq!(hit_ids, ground_truth);
    assert!(
        per_query_us < 500.0,
        "Expected <500us, got {per_query_us}us"
    );
}
