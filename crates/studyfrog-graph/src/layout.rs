//! The layout: positions and velocities as separate arrays (struct of arrays), a fixed spatial
//! grid, and a `step` that allocates nothing.

use crate::params::Params;

/// Largest number of grid cells per side.
const MAX_CELLS_PER_SIDE: usize = 256;

/// Node positions, velocities and the scratch space a step needs, all sized at creation.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// X position of each node.
    pub x: Vec<f32>,
    /// Y position of each node.
    pub y: Vec<f32>,
    /// X velocity.
    pub vx: Vec<f32>,
    /// Y velocity.
    pub vy: Vec<f32>,
    /// Half the width of the square nodes are kept inside.
    pub extent: f32,
    fx: Vec<f32>,
    fy: Vec<f32>,
    cell_of: Vec<u32>,
    cell_start: Vec<u32>,
    cell_items: Vec<u32>,
    cells_per_side: usize,
    cell_size: f32,
}

/// SplitMix64 step, used only to jitter the initial positions reproducibly.
fn mix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn unit(state: &mut u64) -> f32 {
    (mix(state) >> 40) as f32 / (1_u64 << 24) as f32
}

/// A layout of `n` nodes on a golden-angle spiral with a little seeded jitter, so equal seeds give
/// equal layouts and no two nodes start on top of each other.
pub fn layout_new(n: usize, seed: u64, p: &Params) -> Layout {
    let extent = (p.rest_length * (n.max(1) as f32).sqrt() * 2.0).max(p.cutoff * 2.0);
    let cells_per_side = ((2.0 * extent / p.cutoff).ceil() as usize).clamp(1, MAX_CELLS_PER_SIDE);
    let mut rng = seed;
    let (mut x, mut y) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for i in 0..n {
        let angle = i as f32 * 2.399_963;
        let radius = p.rest_length * (i as f32 + 0.5).sqrt() * 0.7;
        x.push(radius * angle.cos() + (unit(&mut rng) - 0.5));
        y.push(radius * angle.sin() + (unit(&mut rng) - 0.5));
    }
    Layout {
        x,
        y,
        vx: vec![0.0; n],
        vy: vec![0.0; n],
        extent,
        fx: vec![0.0; n],
        fy: vec![0.0; n],
        cell_of: vec![0; n],
        cell_start: vec![0; cells_per_side * cells_per_side + 1],
        cell_items: vec![0; n],
        cells_per_side,
        cell_size: 2.0 * extent / cells_per_side as f32,
    }
}

fn cell_coord(l: &Layout, v: f32) -> usize {
    (((v + l.extent) / l.cell_size) as usize).min(l.cells_per_side - 1)
}

/// Sort node indices into grid cells (counting sort into preallocated arrays).
fn bin_nodes(l: &mut Layout) {
    l.cell_start.fill(0);
    for i in 0..l.x.len() {
        let c = (cell_coord(l, l.y[i]) * l.cells_per_side + cell_coord(l, l.x[i])) as u32;
        l.cell_of[i] = c;
        l.cell_start[c as usize + 1] += 1;
    }
    for c in 0..l.cells_per_side * l.cells_per_side {
        l.cell_start[c + 1] += l.cell_start[c];
    }
    // Fill each cell from its start, using cell_start itself as the running cursor, then shift back.
    for i in 0..l.x.len() {
        let c = l.cell_of[i] as usize;
        l.cell_items[l.cell_start[c] as usize] = i as u32;
        l.cell_start[c] += 1;
    }
    for c in (1..=l.cells_per_side * l.cells_per_side).rev() {
        l.cell_start[c] = l.cell_start[c - 1];
    }
    l.cell_start[0] = 0;
}

/// Push apart every pair closer than the cutoff, found through the grid (each pair once).
fn repel(l: &mut Layout, p: &Params) {
    let cutoff2 = p.cutoff * p.cutoff;
    let side = l.cells_per_side;
    for i in 0..l.x.len() {
        let (cx, cy) = (l.cell_of[i] as usize % side, l.cell_of[i] as usize / side);
        for ny in cy.saturating_sub(1)..=(cy + 1).min(side - 1) {
            for nx in cx.saturating_sub(1)..=(cx + 1).min(side - 1) {
                let cell = ny * side + nx;
                for k in l.cell_start[cell]..l.cell_start[cell + 1] {
                    let j = l.cell_items[k as usize] as usize;
                    if j > i {
                        push_pair(l, i, j, cutoff2, p.repulsion);
                    }
                }
            }
        }
    }
}

fn push_pair(l: &mut Layout, i: usize, j: usize, cutoff2: f32, strength: f32) {
    let (dx, dy) = (l.x[i] - l.x[j], l.y[i] - l.y[j]);
    let d2 = dx * dx + dy * dy;
    if d2 >= cutoff2 {
        return;
    }
    // The window (1 - d^2/cutoff^2)^2 takes the force smoothly to zero at the cutoff. A force that
    // jumps to zero there makes nodes near the boundary flicker in and out of range forever.
    let window = 1.0 - d2 / cutoff2;
    let scale = strength / (d2 + 1.0) * window * window;
    let (fx, fy) = (dx * scale, dy * scale);
    l.fx[i] += fx;
    l.fy[i] += fy;
    l.fx[j] -= fx;
    l.fy[j] -= fy;
}

/// Spring force along each edge towards the rest length.
fn attract(l: &mut Layout, edges: &[(u32, u32)], p: &Params) {
    for &(a, b) in edges {
        let (a, b) = (a as usize, b as usize);
        let (dx, dy) = (l.x[b] - l.x[a], l.y[b] - l.y[a]);
        let d = (dx * dx + dy * dy).sqrt().max(0.01);
        let pull = p.spring * (d - p.rest_length) / d;
        l.fx[a] += dx * pull;
        l.fy[a] += dy * pull;
        l.fx[b] -= dx * pull;
        l.fy[b] -= dy * pull;
    }
}

/// Apply forces and gravity, damp, limit the step, move, and keep nodes inside the square.
/// Returns the total kinetic energy after the move.
fn integrate(l: &mut Layout, p: &Params) -> f32 {
    let mut energy = 0.0;
    for i in 0..l.x.len() {
        let gx = l.fx[i] - p.gravity * l.x[i];
        let gy = l.fy[i] - p.gravity * l.y[i];
        let (mut vx, mut vy) = ((l.vx[i] + gx) * p.damping, (l.vy[i] + gy) * p.damping);
        let speed = (vx * vx + vy * vy).sqrt();
        if speed > p.max_step {
            let k = p.max_step / speed;
            (vx, vy) = (vx * k, vy * k);
        }
        (l.vx[i], l.vy[i]) = (vx, vy);
        l.x[i] = (l.x[i] + vx).clamp(-l.extent, l.extent);
        l.y[i] = (l.y[i] + vy).clamp(-l.extent, l.extent);
        energy += vx * vx + vy * vy;
    }
    energy
}

/// Advance the simulation by one step and return its kinetic energy (it falls towards zero as the
/// layout settles). Allocates nothing.
pub fn layout_step(l: &mut Layout, edges: &[(u32, u32)], p: &Params) -> f32 {
    l.fx.fill(0.0);
    l.fy.fill(0.0);
    bin_nodes(l);
    repel(l, p);
    attract(l, edges, p);
    integrate(l, p)
}
