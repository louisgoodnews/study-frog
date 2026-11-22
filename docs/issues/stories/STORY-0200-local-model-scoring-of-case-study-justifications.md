---
id: STORY-0200
type: story
title: "Local-model scoring of case-study justifications, with the rubric prompt template"
status: done
epic: EPIC-061
parents: ['EPIC-061']
estimate: 3
labels: [m14, slm, rehearse, ui]
acceptance_criteria:
  - 'A local model checks a case justification against the rubric, and its JSON reply replaces the keyword check'
  - 'A remote model is never sent the student''s text for a case'
  - 'An unreadable or missing reply keeps the keyword check and says why'
  - 'The prompt template asks for the choice first, then each rubric point, and a caveats field'
---

## Context
STORY-0180 scored answers with a keyword check only, and the case view did not call a model. The student's justification is personal text, so any model used must run on this computer.

## Description
- Prompt template (`studyfrog-agent::rubric_eval::case_system`): two stages in one reply (choice, then each criterion against the justification alone), ending in one JSON object with `choice_valid`, `criteria_results` and `caveats`.
- Scoring (`setup::case`): "Check my answer" shows the keyword check at once, then sends the case to a selected local provider (`ChatPurpose::Case`). The reply is read by `parse_model_report`; the model's result replaces the keyword check.
- Caveats: shown under the report, from the model or from the keyword check's limits.

## Acceptance Criteria
- [x] A local model checks a case justification against the rubric, and its JSON reply replaces the keyword check
- [x] A remote model is never sent the student's text for a case
- [x] An unreadable or missing reply keeps the keyword check and says why
- [x] The prompt template asks for the choice first, then each rubric point, and a caveats field

Audit (2026-11-22): all four criteria MET, with a local stub endpoint on 127.0.0.1 (no real model is installed; the stub returns JSON, so model quality is NOT CHECKABLE here). (1) A local provider's JSON replaced the keyword check: the source line reads "Checked by the local model" and the model's feedback shows (`/tmp/claude-0/audit/g3/08-model-good.png`, `/tmp/claude-0/audit/g3/10-model-caveat.png`). (2) A remote provider (`https://api.example.com/v1`, allowed) was not sent the text: the view says "Only a local model checks the justification, so the keyword check is shown." and no model call is pending (`/tmp/claude-0/audit/g3/12-model-remote.png`; `submit` returns no `AgentChat` effect for a remote provider). (3) A reply with no JSON kept the keyword check and said why: "The local model could not check it (the reply had no JSON object); the keyword check is shown." (`/tmp/claude-0/audit/g3/11-model-garbage.png`). (4) The prompt (`rubric_eval::case_system`) asks for the choice first, then each criterion against the justification alone, and ends with a JSON object with `caveats` (code read).

## Design Notes
The template runs in one reply, not two calls: the stages are instructions inside it. A second call per criterion would cost more and was not needed for the parse. The keyword check remains the fallback when no local model is selected.

## Dependencies
EPIC-061, STORY-0180, STORY-0181

## Open Questions
- Model quality on small local models is unmeasured; the caveats field is the guard until it is tested against a set of answers.
