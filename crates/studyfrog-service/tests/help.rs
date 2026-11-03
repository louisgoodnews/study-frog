#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The supplied help pages, their links and search, and the user's pages in `wiki/`.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use studyfrog_service::help::{
    PAGES, PageRef, UserPage, broken_links, resolve, search, supplied, title_of,
};
use studyfrog_service::help_user::{
    delete_user_page, free_name, list_user_pages, name_problem, save_user_page,
};
use studyfrog_service::session::open_vault;

fn help_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/help")
}

fn temp(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("studyfrog-help-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn no_supplied_page_links_to_a_page_that_does_not_exist() {
    assert_eq!(broken_links(), Vec::<(&str, String)>::new());
}

#[test]
fn every_page_has_a_unique_id_and_a_title_and_every_file_is_compiled_in() {
    let ids: HashSet<&str> = PAGES.iter().map(|p| p.id).collect();
    assert_eq!(ids.len(), PAGES.len(), "ids are unique");
    for p in PAGES {
        assert!(p.text.starts_with("# "), "{} starts with its title", p.id);
        assert_ne!(title_of(p.text, ""), "", "{} has a title", p.id);
    }
    let readme = fs::read_to_string(help_dir().join("README.md")).unwrap();
    for entry in fs::read_dir(help_dir()).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_suffix(".md").filter(|n| *n != "README") else {
            continue;
        };
        assert!(ids.contains(id), "docs/help/{name} is in the page table");
    }
    for id in &ids {
        assert!(readme.contains(&format!("`{id}`")), "README lists {id}");
    }
}

#[test]
fn the_first_pages_of_the_story_exist() {
    for id in [
        "why-studyfrog",
        "getting-started",
        "faq",
        "glossary",
        "help",
    ] {
        assert!(supplied(id).is_some(), "{id}");
    }
}

fn own(file: &str, text: &str) -> UserPage {
    UserPage {
        file: file.into(),
        text: text.into(),
    }
}

#[test]
fn a_link_finds_a_supplied_id_a_user_file_or_a_title() {
    let user = [own("exam tricks", "# Tricks before exams\nSleep.")];
    assert_eq!(
        resolve("library", &user),
        Some(PageRef::Supplied("library".into()))
    );
    assert_eq!(
        resolve("Exam Tricks", &user),
        Some(PageRef::User("exam tricks".into()))
    );
    assert_eq!(
        resolve("tricks before exams", &user),
        Some(PageRef::User("exam tricks".into()))
    );
    let faq_title = title_of(supplied("faq").unwrap().text, "").to_owned();
    assert_eq!(
        resolve(&faq_title, &[]),
        Some(PageRef::Supplied("faq".into()))
    );
    assert_eq!(resolve("no such page", &user), None);
}

#[test]
fn search_puts_title_matches_first_and_covers_user_pages() {
    let user = [own("mine", "# Mine\nSomething about the calendar and me.")];
    let hits = search("calendar", &user);
    assert!(!hits.is_empty());
    assert_eq!(hits[0].page, PageRef::Supplied("calendar".into()));
    assert!(hits.iter().any(|h| h.page == PageRef::User("mine".into())));
    let mine = hits.iter().find(|h| h.title == "Mine").unwrap();
    assert!(mine.snippet.contains("calendar"));
    assert!(search("   ", &user).is_empty());
    assert!(search("zzzz-nothing-zzzz", &user).is_empty());
}

#[test]
fn user_pages_are_written_renamed_listed_and_deleted_in_wiki() {
    let root = temp("crud");
    assert_eq!(list_user_pages(&root).unwrap(), Vec::new());
    let pages = save_user_page(&root, None, "Exam plan", "# Exam plan\n").unwrap();
    assert_eq!(pages, vec![own("Exam plan", "# Exam plan\n")]);
    assert!(root.join("wiki/Exam plan.md").is_file());
    assert!(
        save_user_page(&root, None, "exam plan", "x").is_err(),
        "names differing only in case clash"
    );
    assert!(save_user_page(&root, None, "Exam plan", "again").is_err());
    let pages = save_user_page(&root, Some("Exam plan"), "Plan", "# Plan\n").unwrap();
    assert_eq!(pages, vec![own("Plan", "# Plan\n")]);
    assert!(!root.join("wiki/Exam plan.md").exists());
    assert_eq!(delete_user_page(&root, "Plan").unwrap(), Vec::new());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn page_names_are_checked_and_free_names_are_found() {
    for bad in ["", "  ", ".hidden", "a/b", "a\\b", "c:d", &"x".repeat(81)] {
        assert!(name_problem(bad).is_some(), "{bad:?}");
    }
    assert_eq!(name_problem("My page 2"), None);
    let pages = [own("My page", ""), own("my page 2", "")];
    assert_eq!(free_name("My page", &pages), "My page 3");
    assert_eq!(free_name("Other", &pages), "Other");
}

#[test]
fn the_wiki_folder_is_not_indexed_as_items() {
    let root = temp("index");
    fs::create_dir_all(root.join("wiki")).unwrap();
    fs::write(root.join("wiki/notes.md"), "# Notes\nno front matter\n").unwrap();
    let (_session, info) = open_vault(root.to_str().unwrap()).unwrap();
    assert!(
        info.overview.problems.is_empty(),
        "{:?}",
        info.overview.problems
    );
    assert_eq!(info.overview.inbox_items, 0);
    assert_eq!(list_user_pages(&root).unwrap().len(), 1);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn help_pages_join_the_assistant_context_within_the_budget_and_page_changes_apply() {
    use studyfrog_agent::wiki::Change;
    use studyfrog_service::agent_help::{apply_page, with_help_pages};
    use studyfrog_service::agent_vault::{CONTEXT_BUDGET, WikiContext};
    let pages = [
        own("Plan", "# Plan\nRead daily."),
        own("Big", &"x".repeat(10)),
    ];
    let ctx = with_help_pages(WikiContext::default(), &pages);
    assert_eq!(ctx.items.len(), 2);
    assert_eq!(ctx.items[0].uid, "help:Plan");
    assert_eq!(ctx.items[0].title, "Plan");
    let full = WikiContext {
        chars: CONTEXT_BUDGET,
        ..WikiContext::default()
    };
    assert_eq!(with_help_pages(full, &pages).left_out, 2);

    let root = temp("agent");
    let create = Change::CreatePage {
        name: "Plan".into(),
        text: "# Plan".into(),
    };
    apply_page(&root, &create).unwrap();
    assert!(apply_page(&root, &create).is_err(), "never overwrites");
    let update = Change::UpdatePage {
        name: "Plan".into(),
        text: "# Plan\nv2".into(),
    };
    apply_page(&root, &update).unwrap();
    assert_eq!(list_user_pages(&root).unwrap()[0].text, "# Plan\nv2");
    let gone = Change::UpdatePage {
        name: "Gone".into(),
        text: "x".into(),
    };
    assert!(apply_page(&root, &gone).is_err());
    assert!(!root.join("wiki/Gone.md").exists());
    let _ = fs::remove_dir_all(&root);
}
