#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom rail (STORY-0230): it switches between the session form and the saved sessions, and
//! its two buttons are on screen before a session starts.
use classroom_common::{classroom, sized};
use common::flow::opened;
use egui_kittest::kittest::Queryable;
use studyfrog_ui::classroom_types::{ClassroomMsg, ClassroomSection};
use studyfrog_ui::state::Screen;

mod classroom_common;
mod common;

#[test]
fn the_classroom_rail_switches_between_set_up_and_saved_sessions() {
    let (_vault, mut m, _w) = opened("rail-classroom");
    assert_eq!(m.state.classroom.section, ClassroomSection::Setup);
    classroom(&mut m, ClassroomMsg::Section(ClassroomSection::Saved));
    assert_eq!(m.state.classroom.section, ClassroomSection::Saved);
}

#[test]
fn the_classroom_rail_shows_both_of_its_buttons_before_a_session_starts() {
    let (_vault, mut m, _w) = opened("rail-classroom-view");
    m.state.screen = Screen::Classroom;
    let mut h = sized(m, 1000.0, 720.0);
    h.run_steps(3);
    assert!(h.query_by_label("Set up").is_some());
    assert!(h.query_by_label("Saved sessions").is_some());
}

#[test]
fn the_graph_rail_switches_between_the_map_and_the_list() {
    use studyfrog_store::overview::LinkGraph;
    use studyfrog_ui::model::apply;
    use studyfrog_ui::msg::{Msg, MsgGraph, graph::GraphSection};

    let (_vault, mut m, _w) = opened("rail-graph");
    apply(
        &mut m,
        Msg::Graph(MsgGraph::GraphLoaded(Ok(LinkGraph::default()))),
    );
    assert_eq!(m.state.graph.as_ref().unwrap().section, GraphSection::Map);
    apply(&mut m, Msg::Graph(MsgGraph::Section(GraphSection::List)));
    assert_eq!(m.state.graph.as_ref().unwrap().section, GraphSection::List);
}
