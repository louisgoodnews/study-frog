use crate::plan::{
    Order, ScheduleEffect, effect_from_name, effect_name, order_from_name, order_name,
    plan_hardest, plan_sequential, plan_shuffled, plan_spaced, plan_stalest,
};

const ORDERS: [Order; 7] = [
    Order::Due,
    Order::Sequential,
    Order::Shuffled,
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
