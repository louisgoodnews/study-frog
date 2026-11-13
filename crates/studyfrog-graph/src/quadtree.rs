//! 2D spatial quadtree indexing and viewport culling.
//!
//! Plain data representations of spatial rectangles, items, and quadtrees.
//! Adheres strictly to Principle P1: zero inherent impl blocks, all domain logic
//! lives in pure free functions.

/// Axis-aligned 2D rectangle in top-left coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: f64,
    /// Y coordinate of the top-left corner.
    pub y: f64,
    /// Width of the rectangle (must be non-negative).
    pub w: f64,
    /// Height of the rectangle (must be non-negative).
    pub h: f64,
}

/// A spatial item storing an identifier, bounding rectangle, and arbitrary payload.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialItem<T> {
    /// Unique item identifier.
    pub id: String,
    /// Bounding rectangle in spatial coordinates.
    pub rect: Rect,
    /// User payload attached to this spatial item.
    pub payload: T,
}

/// A 2D spatial quadtree index for fast range queries and viewport culling.
#[derive(Debug, Clone, PartialEq)]
pub struct Quadtree<T> {
    /// Bounding rectangle of this quadtree node.
    pub bounds: Rect,
    /// Maximum number of items in a leaf before subdividing.
    pub capacity: usize,
    /// Maximum remaining tree depth for recursive subdivision.
    pub max_depth: usize,
    /// Items contained directly within this node.
    pub items: Vec<SpatialItem<T>>,
    /// Child quadrants [NW, NE, SW, SE], or None if leaf.
    pub children: Option<Box<[Quadtree<T>; 4]>>,
}

/// Creates a new rectangle with top-left origin `(x, y)` and size `(w, h)`.
pub fn new_rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect { x, y, w, h }
}

/// Creates a new spatial item.
pub fn new_spatial_item<T>(id: impl Into<String>, rect: Rect, payload: T) -> SpatialItem<T> {
    SpatialItem {
        id: id.into(),
        rect,
        payload,
    }
}

/// Returns true if two axis-aligned rectangles intersect (inclusive of touching boundaries).
pub fn rect_intersects(a: &Rect, b: &Rect) -> bool {
    if a.w < 0.0 || a.h < 0.0 || b.w < 0.0 || b.h < 0.0 {
        return false;
    }
    a.x <= b.x + b.w && a.x + a.w >= b.x && a.y <= b.y + b.h && a.y + a.h >= b.y
}

/// Returns true if the rectangle contains the given 2D point (inclusive of boundaries).
pub fn rect_contains_point(r: &Rect, px: f64, py: f64) -> bool {
    if r.w < 0.0 || r.h < 0.0 {
        return false;
    }
    px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h
}

/// Returns true if `outer` completely contains `inner` (inclusive of boundaries).
pub fn rect_contains_rect(outer: &Rect, inner: &Rect) -> bool {
    if outer.w < 0.0 || outer.h < 0.0 || inner.w < 0.0 || inner.h < 0.0 {
        return false;
    }
    inner.x >= outer.x
        && inner.x + inner.w <= outer.x + outer.w
        && inner.y >= outer.y
        && inner.y + inner.h <= outer.y + outer.h
}

/// Creates an empty quadtree covering `bounds` with the given capacity and max depth.
pub fn new_quadtree<T>(bounds: Rect, capacity: usize, max_depth: usize) -> Quadtree<T> {
    Quadtree {
        bounds,
        capacity,
        max_depth,
        items: Vec::new(),
        children: None,
    }
}

/// Inserts a spatial item into the quadtree.
///
/// Returns `true` if the item was successfully inserted, or `false` if it falls outside
/// the quadtree bounds or has invalid dimensions.
pub fn insert<T>(tree: &mut Quadtree<T>, item: SpatialItem<T>) -> bool {
    if !rect_contains_rect(&tree.bounds, &item.rect) {
        return false;
    }

    if let Some(children) = &mut tree.children {
        if let Some(child) = children
            .iter_mut()
            .find(|c| rect_contains_rect(&c.bounds, &item.rect))
        {
            return insert(child, item);
        }
        tree.items.push(item);
        return true;
    }

    tree.items.push(item);

    if tree.items.len() > tree.capacity && tree.max_depth > 0 {
        subdivide(tree);
    }

    true
}

/// Subdivides a leaf node into 4 quadrant children [NW, NE, SW, SE] and pushes down items.
fn subdivide<T>(tree: &mut Quadtree<T>) {
    let hw = tree.bounds.w / 2.0;
    let hh = tree.bounds.h / 2.0;
    let x = tree.bounds.x;
    let y = tree.bounds.y;
    let next_depth = tree.max_depth.saturating_sub(1);

    let mut children = Box::new([
        new_quadtree(Rect { x, y, w: hw, h: hh }, tree.capacity, next_depth),
        new_quadtree(
            Rect {
                x: x + hw,
                y,
                w: hw,
                h: hh,
            },
            tree.capacity,
            next_depth,
        ),
        new_quadtree(
            Rect {
                x,
                y: y + hh,
                w: hw,
                h: hh,
            },
            tree.capacity,
            next_depth,
        ),
        new_quadtree(
            Rect {
                x: x + hw,
                y: y + hh,
                w: hw,
                h: hh,
            },
            tree.capacity,
            next_depth,
        ),
    ]);

    let old_items = std::mem::take(&mut tree.items);
    for it in old_items {
        if let Some(child) = children
            .iter_mut()
            .find(|c| rect_contains_rect(&c.bounds, &it.rect))
        {
            insert(child, it);
        } else {
            tree.items.push(it);
        }
    }
    tree.children = Some(children);
}

/// Queries all spatial items whose bounding rectangles intersect `range`.
///
/// Appends references to matching items into `results` without allocating tree nodes.
pub fn query_range<'a, T>(
    tree: &'a Quadtree<T>,
    range: &Rect,
    results: &mut Vec<&'a SpatialItem<T>>,
) {
    if !rect_intersects(&tree.bounds, range) {
        return;
    }

    for item in &tree.items {
        if rect_intersects(&item.rect, range) {
            results.push(item);
        }
    }

    if let Some(children) = &tree.children {
        for child in children.iter() {
            query_range(child, range, results);
        }
    }
}

/// Queries all spatial items whose bounding rectangles contain the 2D point `(px, py)`.
///
/// Appends references to matching items into `results` without allocating tree nodes.
pub fn query_point<'a, T>(
    tree: &'a Quadtree<T>,
    px: f64,
    py: f64,
    results: &mut Vec<&'a SpatialItem<T>>,
) {
    if !rect_contains_point(&tree.bounds, px, py) {
        return;
    }

    for item in &tree.items {
        if rect_contains_point(&item.rect, px, py) {
            results.push(item);
        }
    }

    if let Some(children) = &tree.children {
        for child in children.iter() {
            query_point(child, px, py, results);
        }
    }
}

/// Clears all items and collapses all child quadrants, leaving the tree empty.
pub fn clear<T>(tree: &mut Quadtree<T>) {
    tree.items.clear();
    tree.children = None;
}

/// Returns the total number of items stored in this node and all descendant quadrants.
pub fn count<T>(tree: &Quadtree<T>) -> usize {
    let mut total = tree.items.len();
    if let Some(children) = &tree.children {
        for child in children.iter() {
            total += count(child);
        }
    }
    total
}
