//! Stable item identity.

/// Stable, persisted identity of a stack or item: the 128 bits of the ULID stored in
/// front-matter. Parsing and formatting live in `studyfrog-md`; this crate only carries the bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(pub u128);
