//! Achievement engine: rules evaluated over review history. Pure; the service layer reads the
//! history and remembers which achievements were already unlocked.

pub mod defaults;
pub mod evaluate;
pub mod stats;
