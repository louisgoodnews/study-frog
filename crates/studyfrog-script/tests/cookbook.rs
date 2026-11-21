#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript cookbook (STORY-0172): every recipe parses, the read-only ones run on an empty engine
//! and write nothing, and there are at least eight.

use std::collections::BTreeSet;

use studyfrog_script::cookbook::{RECIPES, recipe};
use studyfrog_script::{parse_frogscript, run_frogscript};

/// Verbs and calls that change items or the vault.
const WRITES: [&str; 8] = [
    "| set ",
    "| tag ",
    "| untag ",
    "create_card",
    "schedule_study",
    "add_entry",
    "diary.entry",
    "export ",
];

#[test]
fn there_are_at_least_eight_recipes_with_unique_names() {
    assert!(RECIPES.len() >= 8, "{} recipes", RECIPES.len());
    let names: BTreeSet<&str> = RECIPES.iter().map(|r| r.name).collect();
    assert_eq!(names.len(), RECIPES.len(), "names are unique");
    for r in RECIPES {
        assert!(!r.summary.is_empty() && r.summary.len() < 160, "{}", r.name);
        assert!(
            r.code.contains('#'),
            "{} explains itself in a comment",
            r.name
        );
    }
}

#[test]
fn every_recipe_parses() {
    for r in RECIPES {
        parse_frogscript(r.code).unwrap_or_else(|e| panic!("{} does not parse: {e:?}", r.name));
    }
}

#[test]
fn read_only_recipes_run_on_an_empty_vault_and_write_nothing() {
    for r in RECIPES.iter().filter(|r| r.read_only) {
        for w in WRITES {
            assert!(
                !r.code.contains(w),
                "{} is read-only but contains {w}",
                r.name
            );
        }
        // `vault` and `calendar` are provided by the app's run, not by the bare engine: those recipes are
        // run through the app in its own tests.
        if r.code.contains("vault.") || r.code.contains("calendar.") {
            continue;
        }
        run_frogscript(r.code, Vec::new(), 1_800_000_000_000)
            .unwrap_or_else(|e| panic!("{} fails to run: {e:?}", r.name));
    }
}

#[test]
fn a_recipe_is_found_by_its_name() {
    assert!(recipe("Due today").is_some());
    assert!(recipe("No such recipe").is_none());
    assert!(
        RECIPES.iter().any(|r| !r.read_only),
        "some recipes change items"
    );
}
