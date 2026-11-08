//! The plain data of the deck simulation (STORY-0105): cards, how the learner rates and paces, the setup and
//! the counts of a day.

use studyfrog_core::memory::{MemState, Sm2State};

/// What a scheduler knows about a card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimMemory {
    /// FSRS state.
    Fsrs(MemState),
    /// SM-2 state.
    Sm2(Sm2State),
}

/// A card of the deck.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimCard {
    /// Its state, `None` while it is new.
    pub memory: Option<SimMemory>,
    /// Day of its last review.
    pub last_day: i64,
    /// Day it is due.
    pub due_day: i64,
    /// Its last rating was Again.
    pub relearning: bool,
    /// A new card is scheduled with SM-2 (else FSRS).
    pub sm2: bool,
    /// Index of its pace in [`SimSetup::paces`].
    pub pace: u16,
}

/// How the learner rates: the share of first reviews forgotten, and among recalled reviews the share of Hard and
/// of Good (the rest is Easy).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatingMix {
    /// First reviews rated Again.
    pub first_again: f32,
    /// Recalled reviews rated Hard.
    pub hard: f32,
    /// Recalled reviews rated Good.
    pub good: f32,
}

/// Seconds a review takes by stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    /// A card seen for the first time.
    pub new_s: f32,
    /// A card relearned after a lapse.
    pub learn_s: f32,
    /// Any other review.
    pub review_s: f32,
}

/// Paces used when there is no history: 20, 15 and 10 seconds.
pub fn pace_default() -> Pace {
    Pace {
        new_s: 20.0,
        learn_s: 15.0,
        review_s: 10.0,
    }
}

/// How the future is simulated.
#[derive(Debug, Clone, PartialEq)]
pub struct SimSetup {
    /// The first day.
    pub today: i64,
    /// Days simulated.
    pub days: usize,
    /// Futures simulated (for the spread).
    pub trajectories: u32,
    /// Starts the random sequence.
    pub seed: u64,
    /// Chance a day is missed (its reviews move to the next day).
    pub miss_probability: f32,
    /// New cards started per day (fractions add up over days).
    pub new_per_day: f32,
    /// New cards are started only on the first this many days.
    pub new_days: usize,
    /// How the learner rates.
    pub mix: RatingMix,
    /// Paces, indexed by [`SimCard::pace`] (an empty list means the defaults).
    pub paces: Vec<Pace>,
}

/// Reviews of one simulated day by stage, and the time they take.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DayCount {
    /// First reviews.
    pub new: u32,
    /// Reviews after a lapse.
    pub learning: u32,
    /// Reviews of cards with stability under 21 days.
    pub young: u32,
    /// Reviews of cards with stability of 21 days or more.
    pub mature: u32,
    /// Seconds of study.
    pub seconds: f32,
}

/// Total reviews of a day.
pub fn day_total(d: &DayCount) -> u32 {
    d.new + d.learning + d.young + d.mature
}
