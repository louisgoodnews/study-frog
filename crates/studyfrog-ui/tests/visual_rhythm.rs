#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0115: the spacing rhythm obeys the rule of halves at every density (tokens, egui's style and the drawn
//! Settings page), controls are drawn by fill without outlines unless high contrast is on, and every screen shows at
//! most one accent-filled primary action.

mod common;

use common::views::{harness, open_model, running_model};
use eframe::egui;
use egui_kittest::kittest::Queryable;
use studyfrog_core::appearance::{Appearance, Density};
use studyfrog_core::run::RunMode;
use studyfrog_ui::app_settings_ui::{SettingsTab, SettingsTabMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};
use studyfrog_ui::theme::{Look, palette_with};
use studyfrog_ui::theme_apply::apply_look;
use studyfrog_ui::theme_space::{primaries, rhythm};

const DENSITIES: [Density; 3] = [Density::Compact, Density::Comfortable, Density::Spacious];

#[test]
fn the_tokens_obey_the_rule_of_halves_at_every_density() {
    for d in DENSITIES {
        let r = rhythm(d);
        assert!(
            r.tight < r.item && r.item < r.card && r.card < r.section,
            "{d:?}: {r:?}"
        );
        assert_eq!(r.card, 2.0 * r.item, "{d:?}");
        assert_eq!(r.item, 2.0 * r.tight, "{d:?}");
        let ctx = egui::Context::default();
        apply_look(
            &ctx,
            &Appearance {
                density: d,
                ..Appearance::default()
            },
        );
        let style = ctx.style_of(egui::Theme::Dark);
        assert_eq!(
            style.spacing.item_spacing,
            egui::vec2(r.item, r.item),
            "{d:?}"
        );
    }
    assert_eq!(rhythm(Density::Comfortable).section, 24.0);
}

#[test]
fn controls_have_no_outline_unless_high_contrast_is_on() {
    for high_contrast in [false, true] {
        let ctx = egui::Context::default();
        apply_look(
            &ctx,
            &Appearance {
                high_contrast,
                ..Appearance::default()
            },
        );
        for (theme, dark) in [(egui::Theme::Dark, true), (egui::Theme::Light, false)] {
            let p = palette_with(
                dark,
                Look {
                    high_contrast,
                    ..Look::default()
                },
            );
            let w = &ctx.style_of(theme).visuals.widgets;
            let edge = if high_contrast {
                p.border
            } else {
                w.inactive.weak_bg_fill
            };
            assert_eq!(
                w.inactive.bg_stroke.color, edge,
                "hc {high_contrast}, dark {dark}"
            );
            assert_eq!(
                w.hovered.bg_stroke.color, p.accent,
                "hover shows an outline"
            );
            assert_eq!(
                w.noninteractive.bg_stroke.color, p.border,
                "separators stay"
            );
        }
    }
}

#[test]
fn the_settings_page_keeps_siblings_closer_than_padding_and_padding_closer_than_groups() {
    let mut m = open_model();
    m.state.toasts.clear();
    apply(&mut m, Msg::Navigate(Screen::Settings));
    apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::Select(SettingsTab::Appearance)),
    );
    let mut h = harness(m);
    h.set_size(egui::vec2(1400.0, 1000.0));
    h.run();
    let rect = |label: &str| h.get_by_label(label).rect();
    let r = rhythm(Density::Comfortable);
    // Siblings: the theme buttons in a row.
    let sibling = rect("Theme: Light").left() - rect("Theme: Dark").right();
    // Inside a card: from the caption to the first control is the tight gap plus the card's padding.
    let inside = rect("Theme: Dark").top() - rect("Theme").bottom() - r.tight;
    // Between groups: from the last line of one card to the next caption.
    let last = h
        .get_by_label_contains("Follow system uses the theme")
        .rect();
    let between = rect("Size and density").top() - last.bottom() - inside;
    assert!((sibling - r.item).abs() < 1.5, "sibling gap {sibling}");
    assert!((inside - r.card).abs() < 1.5, "card padding {inside}");
    assert!((between - r.section).abs() < 1.5, "group gap {between}");
    assert!(sibling < inside && inside < between);
}

fn count_on(m: Model) -> usize {
    let mut h = harness(m);
    h.set_size(egui::vec2(1400.0, 900.0));
    h.run();
    primaries(&h.ctx).1
}

#[test]
fn every_screen_has_at_most_one_accent_filled_primary_action() {
    let screens = [
        Screen::Home,
        Screen::Library,
        Screen::Rehearse,
        Screen::Settings,
        Screen::Help,
        Screen::Calendar,
        Screen::Board,
        Screen::Reports,
        Screen::Diary,
        Screen::Agent,
        Screen::Prompts,
        Screen::Achievements,
        Screen::Graph,
        Screen::Folders,
    ];
    for s in screens {
        let mut m = open_model();
        m.state.toasts.clear();
        apply(&mut m, Msg::Navigate(s));
        assert!(count_on(m) <= 1, "{s:?} shows more than one primary action");
    }
    let mut stack = open_model();
    apply(&mut stack, Msg::Select(Selection::Stack("fr".into())));
    assert!(count_on(stack) <= 1, "the stack page");
    assert!(count_on(running_model(RunMode::Flip)) <= 1, "a run");
    assert!(count_on(running_model(RunMode::Answer)) <= 1, "a typed run");
}
