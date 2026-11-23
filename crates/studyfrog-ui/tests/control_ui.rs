#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The widget commands of the control protocol (`ui tree`, `ui find`, `ui click`): words, class, the
//! answers from a frame's tree, and the headless refusal (the tree is in the window). EPIC-038, EPIC-064.

mod common;
mod control_common;

use control_common::*;
use eframe::egui::accesskit::{Action, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use studyfrog_ui::control_cmd::ui::UiCmd;
use studyfrog_ui::control_cmd::{Class, Command, class_of};
use studyfrog_ui::control_reply::ui::{click_request, find, find_json, pick, tree_json, widgets};
use studyfrog_ui::control_window::ui::destructive;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

/// A frame: a window with a button "Save", a disabled button "Save", an unnamed group holding "Deep",
/// and a text field with only a value.
fn frame() -> TreeUpdate {
    let node = |role: Role, label: Option<&str>| {
        let mut n = Node::new(role);
        if let Some(l) = label {
            n.set_label(l);
        }
        n
    };
    let mut root = node(Role::Window, None);
    root.set_children(vec![NodeId(2), NodeId(3), NodeId(4), NodeId(5)]);
    let mut off = node(Role::Button, Some("Save"));
    off.set_disabled();
    let mut group = node(Role::Group, None);
    group.set_children(vec![NodeId(6)]);
    let mut field = node(Role::TextInput, None);
    field.set_value("abc");
    TreeUpdate {
        nodes: vec![
            (NodeId(1), root),
            (NodeId(2), node(Role::Button, Some("Save"))),
            (NodeId(3), off),
            (NodeId(4), group),
            (NodeId(5), field),
            (NodeId(6), node(Role::Label, Some("Deep"))),
        ],
        tree: Some(TreeInfo::new(NodeId(1))),
        tree_id: TreeId::ROOT,
        focus: NodeId(1),
    }
}

#[test]
fn ui_words_parse() {
    assert_eq!(cmd("ui"), Command::Ui(UiCmd::Tree));
    assert_eq!(cmd("ui tree"), Command::Ui(UiCmd::Tree));
    assert_eq!(cmd("ui find Save"), Command::Ui(UiCmd::Find("Save".into())));
    assert_eq!(
        cmd("ui click \"Save changes\""),
        Command::Ui(UiCmd::Click("Save changes".into()))
    );
    assert_eq!(
        cmd("ui click Save changes"),
        Command::Ui(UiCmd::Click("Save changes".into()))
    );
    assert!(parse_request("ui click").is_err());
    assert!(parse_request("ui find").is_err());
    assert!(parse_request("ui press Save").is_err());
}

#[test]
fn reading_the_tree_is_read_and_clicking_is_write() {
    assert_eq!(class_of(&cmd("ui tree")), Class::Read);
    assert_eq!(class_of(&cmd("ui find Save")), Class::Read);
    assert_eq!(class_of(&cmd("ui click Save")), Class::Write);
}

#[test]
fn the_tree_lists_the_named_widgets_in_reading_order() {
    let f = frame();
    let named: Vec<(String, String, bool)> = widgets(&f)
        .into_iter()
        .map(|w| (w.role, w.label, w.enabled))
        .collect();
    assert_eq!(
        named,
        vec![
            ("Button".to_owned(), "Save".to_owned(), true),
            ("Button".to_owned(), "Save".to_owned(), false),
            ("Label".to_owned(), "Deep".to_owned(), true),
            ("TextInput".to_owned(), String::new(), true),
        ]
    );
    let answer = tree_json(&f);
    assert_eq!(answer["count"], 4, "{answer}");
    assert_eq!(answer["truncated"], false, "{answer}");
    assert_eq!(answer["widgets"][3]["value"], "abc", "{answer}");
}

#[test]
fn find_matches_exactly_then_by_containing_and_pick_needs_one() {
    let f = frame();
    assert_eq!(find(&f, "save").len(), 2);
    assert_eq!(find(&f, "dee")[0].label, "Deep");
    assert!(find(&f, "nowhere").is_empty());
    assert_eq!(find_json(&f, "Deep").unwrap()["count"], 1);
    assert!(
        find_json(&f, "nowhere")
            .unwrap_err()
            .contains("no widget labelled")
    );
    assert!(pick(&f, "save").unwrap_err().contains("2 widgets"));
    assert_eq!(pick(&f, "Deep").unwrap().id, NodeId(6));
}

#[test]
fn a_click_on_a_deleting_label_needs_the_destructive_flag() {
    assert!(destructive("Delete stack"));
    assert!(destructive("Confirm delete"));
    assert!(!destructive("Save changes"));
}

#[test]
fn a_click_is_an_accesskit_click_on_that_node() {
    let req = click_request(NodeId(6));
    assert_eq!(req.action, Action::Click);
    assert_eq!(req.target_node, NodeId(6));
    assert_eq!(req.target_tree, TreeId::ROOT);
}

#[test]
fn a_headless_instance_refuses_the_widget_tree_and_says_where_it_is() {
    let (vault, logs) = (sample_vault("ctl-ui"), temp_dir("ctl-ui-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "ui tree");
    assert_eq!(r["ok"], false, "{r}");
    assert!(r["error"].as_str().unwrap().contains("window"), "{r}");
}

#[test]
fn a_click_is_refused_when_writing_is_switched_off() {
    let (vault, logs) = (sample_vault("ctl-ui-ro"), temp_dir("ctl-ui-ro-logs"));
    let read_only = ControlPolicy {
        allow_write: false,
        allow_destructive: false,
    };
    let mut e = engine(&vault.path, &logs.path, read_only);
    let r = ask(&mut e, "ui click Save");
    assert_eq!(r["ok"], false, "{r}");
    assert!(
        r["error"]
            .as_str()
            .unwrap()
            .contains("writing is switched off"),
        "{r}"
    );
}
