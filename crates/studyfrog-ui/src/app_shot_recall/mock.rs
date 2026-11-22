//! Screenshot setups for the mock exam: an exam made offline, and a graded one.

use crate::model::Model;
use crate::state::Screen;

/// A mock exam made offline from a short note, shown with its answers hidden.
pub fn mock_demo(model: &mut Model) {
    let notes = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n";
    let chunks = studyfrog_rehearsal::curriculum::chunk_markdown(notes);
    let g = studyfrog_service::exam_gen::generate(&chunks, 6, "Mock exam: bio", None);
    model.state.screen = Screen::Rehearse;
    model.state.rehearse.mock = Some(crate::setup::mock_exam::MockExamUi {
        folder: Some("bio".to_owned()),
        title: "Mock exam: bio".to_owned(),
        result: Some(Box::new(g)),
        ..Default::default()
    });
}

/// A graded mock exam: the first answer complete, the others empty.
pub fn mock_graded(model: &mut Model) {
    mock_demo(model);
    let Some(ui) = model.state.rehearse.mock.as_mut() else {
        return;
    };
    let Some(g) = ui.result.as_ref() else { return };
    let mut answers = vec![String::new(); g.keys.len()];
    if let Some(k) = g.keys.first() {
        answers[0] = format!("{} {}", k.sample_answer, k.rubric.join(" "));
    }
    ui.graded = Some(crate::setup::mock_exam::grading::grade(
        &g.exam, &g.keys, &answers,
    ));
    ui.answers = answers;
}
