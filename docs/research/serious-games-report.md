# Serious games: research report and strategy for StudyFrog

Date: 2026-11-27. Status: draft; effect sizes not yet checked against primary texts. Scope: what the literature says about serious games for learning, and what StudyFrog should do with it.

## 1. Provenance and limits (read first)

- **Google Scholar was not reachable.** A direct request on 2026-11-27 redirected to a Google bot-verification page (HTTP 302 to `google.com/sgsync`). The earlier session's log (2026-11-26) records the same block for five queries. Results below therefore come from web search, OpenAlex, Crossref and ERIC records logged in `docs/research/serious-games.md`, not from Scholar listings.
- **Primary abstracts could not be read.** PsycNet returned a loading shell with no text, and SAGE returned HTTP 403. The effect sizes quoted here come from secondary summaries and must be checked against the papers before they are cited in code, docs or the UI.
- Verification status used below: **[s]** = stated in a secondary summary, not confirmed at the primary text; **[v]** = confirmed in the earlier session's registry records (OpenAlex/Crossref/ERIC).

## 2. Definitions

- A serious game has an explicit educational purpose and is not meant to be played primarily for amusement, but it may still be entertaining (Abt, 1970 [s]).
- Serious games (digital game-based learning) are complete game systems built around learning goals. Gamification adds isolated game elements (points, badges, leaderboards) to non-game tasks (Sailer et al., 2017 [v]; Sailer and Homner, 2020 [v]).
- For StudyFrog the distinction matters: Match mode and Rehearsal are serious-game-like, while achievements, streaks and progress bars are gamification.

## 3. Effectiveness

| Source | Claim | Status |
|---|---|---|
| Wouters et al., 2013, J. Educ. Psychol. | Serious games beat conventional instruction on learning (d ≈ 0.29) and retention (d ≈ 0.36); no reliable motivation advantage (d ≈ 0.26, n.s.). Gains larger with supplementary instruction, multiple sessions and group play. About 39 studies. | [s], registry metadata [v] |
| Clark, Tanner-Smith and Killingsworth, 2016, Rev. Educ. Res. | Games vs non-game instruction g ≈ 0.33 (95% CI 0.19–0.48). Augmented game designs beat standard game designs (g ≈ 0.34). Simpler visuals did as well as complex ones. | [s]; sample counts disagree across summaries (57 vs 69 studies) |
| Ren, Xu and Liu, 2024 | Gamification beat serious games on learning achievement and extrinsic motivation; serious games were better for intrinsic motivation. | [v] per earlier log |
| Healthcare professions meta-analysis, 2021 (Simul Healthc) | Positive effects for serious games in health education. | [s] |
| Retrieval practice, digital flashcards (various, 2024–2026) | Active recall drives the effect; evidence that serious games add benefit beyond flashcards is promising but limited. | [s], abstracts only |
| Gamified feedback in retrieval practice (points, progress bars) | Raised motivation but not learning. | [s] |

**Reading:** the effect is real but moderate (d ≈ 0.3). The most robust levers are design details (supplementary instruction, repeated sessions, feedback and debriefing), not the game surface. Motivation gains are inconsistent, and gamification alone does not reliably improve learning.

## 4. Mechanisms that matter for StudyFrog

1. **Retrieval practice.** Active recall is the core mechanism behind the effect. Rehearsal and Match "Drill missed items" already do this; serious-game framing should wrap retrieval, not replace it.
2. **Feedback and debriefing.** Structured debriefing is a mediating element (2024 evaluation-games framework [s]). Match mode's debrief (`match_victory.rs`, in the uncommitted match work) fits this.
3. **Stealth assessment.** In-game measurement of competence without test anxiety (Shute's ECD work [s]). StudyFrog can infer mastery from play and feed it to scheduling, but must not add visible test pressure.
4. **Game design beyond the medium.** Augmented designs help more than the bare medium. The design question is which mechanic carries the learning, not whether the app is a game.
5. **Motivation is not a given.** Do not claim motivation gains. Gamified feedback can motivate without improving learning, so measure learning outcomes.

## 5. Strategy for StudyFrog

Principles (derived from the above):
- Keep retrieval central. Every serious-game mode must end in a graded recall event that feeds the scheduler.
- Build debriefing into every mode, with a short "what you missed and why" step.
- Measure learning, not only engagement: compare first-try accuracy and retention across sessions per user.
- Avoid pressure mechanics (timers, lives) as the default; stealth assessment should infer mastery quietly.
- Keep gamification (achievements, streaks, the companion frog in EPIC-081) as secondary reward, never as the learning signal.

Concrete steps, mapped to existing work:
- **Match mode (in progress):** keep the mismatch logging and the debrief. Add a retention check: re-test the same pairs after a delay, and report first-try versus retained accuracy.
- **Rehearsal:** add a supplementary-instruction hook: after a missed card, show the source note as the debrief, which matches the "supplementary instruction" moderator.
- **Stealth assessment:** log per-item response time and error type during play, and feed the scheduler. No visible score during play.
- **Evaluation:** add a small in-app study protocol (opt-in) comparing retention with and without a serious-game mode, to test the claim locally rather than cite it.
- **Research to close:** read the full texts of Wouters 2013 and Clark 2016 and verify the effect sizes and sample counts before any citation enters code, docs or UI. Retrieve them through a library or DOI route; Scholar is blocked here.

## 6. Open questions

- Do the effect sizes hold for adult self-directed study, or only for classroom use?
- Does a serious-game mode beat plain retrieval practice (flashcards) for the same time?
- Which moderator matters most for StudyFrog users: group play, supplementary instruction, or repeated sessions?

## 7. Relation to existing documents

- `docs/research/serious-games.md` is the detailed survey with the audit log. This report adds the effectiveness table and the StudyFrog mapping.
- `docs/roadmap/serious-games-strategy.md` holds the wider strategy; the steps above should be checked against it.
