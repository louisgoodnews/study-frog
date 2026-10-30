use crate::plan::{
    ItemKind, Order, ScheduleEffect, StateFilter, effect_from_name, effect_name,
    item_kind_from_name, item_kind_name, order_from_name, order_name, plan_hardest,
    plan_sequential, plan_shuffled, plan_spaced, plan_stalest, state_filter_from_name,
    state_filter_name,
};

const ORDERS: [Order; 6] = [
    Order::Due,
    Order::Sequential,
    Order::HardestFirst,
    Order::EasiestFirst,
    Order::StalestFirst,
    Order::LowestRetrievability,
];

#[test]
fn names_round_trip() {
    for order in ORDERS {
        assert_eq!(order_from_name(order_name(order)), Some(order));
    }
    for e in [ScheduleEffect::UpdateSchedule, ScheduleEffect::PracticeOnly] {
        assert_eq!(effect_from_name(effect_name(e)), Some(e));
    }
    assert_eq!(order_from_name("nope"), None);
    assert_eq!(
        order_from_name("shuffled"),
        Some(Order::Sequential),
        "runs stored before shuffling became a switch"
    );
}

#[test]
fn standard_plans_have_the_documented_defaults() {
    let spaced = plan_spaced();
    assert_eq!(
        (spaced.order, spaced.effect, spaced.requeue_again),
        (Order::Due, ScheduleEffect::UpdateSchedule, true)
    );
    for p in [
        plan_sequential(),
        plan_shuffled(7),
        plan_hardest(),
        plan_stalest(),
    ] {
        assert_eq!(p.effect, ScheduleEffect::PracticeOnly);
    }
    assert_eq!(plan_shuffled(7).seed, 7);
    assert!(!plan_sequential().requeue_again && plan_hardest().requeue_again);
}

#[test]
fn kind_state_and_scheduler_names_round_trip() {
    use crate::scheduler::{Scheduler, scheduler_from_name, scheduler_name};
    for k in [ItemKind::Note, ItemKind::Card, ItemKind::Question] {
        assert_eq!(item_kind_from_name(item_kind_name(k)), Some(k));
    }
    for s in [
        StateFilter::Any,
        StateFilter::New,
        StateFilter::Learned,
        StateFilter::Due,
        StateFilter::NotDue,
    ] {
        assert_eq!(state_filter_from_name(state_filter_name(s)), Some(s));
    }
    for s in [Scheduler::Fsrs, Scheduler::Sm2] {
        assert_eq!(scheduler_from_name(scheduler_name(s)), Some(s));
    }
    assert_eq!(
        (
            item_kind_from_name("stack"),
            state_filter_from_name("x"),
            scheduler_from_name("x")
        ),
        (None, None, None)
    );
}

#[test]
fn shuffling_is_a_switch_that_leaves_the_order_and_effect_alone() {
    let p = plan_shuffled(3);
    assert_eq!((p.order, p.shuffle, p.seed), (Order::Sequential, true, 3));
    assert_eq!(
        p.effect,
        ScheduleEffect::PracticeOnly,
        "everything-shuffled is a practice run"
    );
    assert!(!plan_spaced().shuffle && !plan_sequential().shuffle);
    let shuffled_spaced = crate::plan::RunPlan {
        shuffle: true,
        ..plan_spaced()
    };
    assert_eq!(
        shuffled_spaced.effect,
        ScheduleEffect::UpdateSchedule,
        "a shuffled spaced run still updates the schedule"
    );
}
