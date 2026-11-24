//! Checking a case-study answer: the keyword check at once, and the local model's check when one is
//! selected (STORY-0200). Pure: the effect asks for the model, and its reply comes back as a message.
use crate::effect::EffectAssistant;

use studyfrog_agent::provider::{Message, Role, is_local};
use studyfrog_agent::rubric_eval::{case_system, fallback_evaluate, json_span, parse_model_report};
use studyfrog_rehearsal::case_study::choice_is_correct;

use super::{CaseUi, picked_names, with_case};
use crate::agent_conn::selected_provider;
use crate::agent_types::ChatPurpose;
use crate::effect::Effect;
use crate::state::State;

/// Check the answer. The keyword check gives a report at once. A selected local model is asked to check
/// the justification as well; a remote one is not, because the answer is the student's own text.
pub(super) fn submit(state: State) -> (State, Vec<Effect>) {
    let Some(cs) = state.rehearse.case_study.clone() else {
        return (state, Vec::new());
    };
    let Some((_, case)) = cs.cases.get(cs.current) else {
        return (state, Vec::new());
    };
    let picked = picked_names(&cs);
    let valid = choice_is_correct(case, &picked);
    let keyword = fallback_evaluate(&cs.rationale, valid, &case.rubric_criteria);
    let base = CaseUi {
        report: Some(keyword),
        checked_by_model: false,
        note: None,
        pending: false,
        ..cs.clone()
    };
    let provider = selected_provider(&state.agent);
    match provider {
        None => (with_case(state, Some(base)), Vec::new()),
        Some(p) if !is_local(&p) => {
            let note =
                "Only a local model checks the justification, so the keyword check is shown.";
            let cs = CaseUi {
                note: Some(note.to_owned()),
                ..base
            };
            (with_case(state, Some(cs)), Vec::new())
        }
        Some(p) => {
            let system = case_system(
                &case.vignette,
                &case.decision_prompt,
                &case.options,
                &case.expected_choices,
                &case.rubric_criteria,
            );
            let text = format!(
                "Chosen: {}\n\nJustification:\n{}",
                picked.join(", "),
                cs.rationale
            );
            let messages = vec![Message {
                role: Role::User,
                content: text,
            }];
            let effect = Effect::Assistant(EffectAssistant::AgentChat {
                provider: p,
                system,
                messages,
                purpose: ChatPurpose::Case,
            });
            let cs = CaseUi {
                pending: true,
                ..base
            };
            (with_case(state, Some(cs)), vec![effect])
        }
    }
}

/// The local model's answer replaces the keyword check when it can be read; otherwise the keyword check
/// stays and a note says why.
pub(super) fn model_replied(cs: CaseUi, result: Result<String, String>) -> CaseUi {
    let Some((_, case)) = cs.cases.get(cs.current) else {
        return CaseUi {
            pending: false,
            ..cs
        };
    };
    let rubric = case.rubric_criteria.clone();
    let outcome = result.and_then(|text| {
        let json = json_span(&text).ok_or_else(|| "the reply had no JSON object".to_owned())?;
        parse_model_report(json, &rubric)
    });
    match outcome {
        Ok(report) => CaseUi {
            report: Some(report),
            checked_by_model: true,
            pending: false,
            note: None,
            ..cs
        },
        Err(e) => CaseUi {
            pending: false,
            note: Some(format!(
                "The local model could not check it ({e}); the keyword check is shown."
            )),
            ..cs
        },
    }
}
