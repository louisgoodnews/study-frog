//! Layout parameters.

/// Tunables of the force simulation. All lengths are in layout units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// Strength of the repulsion between nearby nodes.
    pub repulsion: f32,
    /// Spring constant of an edge.
    pub spring: f32,
    /// Length at which an edge pulls nothing.
    pub rest_length: f32,
    /// Pull of every node towards the centre.
    pub gravity: f32,
    /// Velocity kept per step (0 to 1).
    pub damping: f32,
    /// Nodes further apart than this do not repel each other (this is what makes a step cheap).
    pub cutoff: f32,
    /// Largest distance a node may move in one step.
    pub max_step: f32,
}

/// Sensible defaults for a few thousand nodes.
pub fn params_default() -> Params {
    Params {
        repulsion: 40.0,
        spring: 0.04,
        rest_length: 18.0,
        gravity: 0.004,
        damping: 0.82,
        cutoff: 60.0,
        max_step: 12.0,
    }
}
