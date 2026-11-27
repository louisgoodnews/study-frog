# UI and UX for learning: evidence report for StudyFrog

Status: research draft, 2026-11-27. Builds on [ui-aesthetics-and-ux-science](ui-aesthetics-and-ux-science.md) (colour, typography, Gestalt grouping) and [learning-science](learning-science.md). This report covers what that note does not: how the interface should carry instruction (cognitive load, multimedia design), feedback and retrieval interaction, motion, analytics dashboards, motor accessibility and display mode. The strategy derived from it is in [ui-strategy](../design/ui-strategy.md), and the mockups are in [docs/design/mockups](../design/mockups/index.html).

## 1. Provenance and confidence

- **Google Scholar was not reachable** from this environment (requests redirect to a bot-verification page). Searches used general web search, with several W3C and publisher pages read directly. Primary full texts were mostly not read.
- Each claim is tagged:
  - **[P]** primary or standards source read directly (W3C WCAG pages);
  - **[S]** secondary summary of a named study, as returned by search, not checked against the original;
  - **[X]** source found but its figures conflict or are unclear across summaries.
- Effect sizes below are **[S]** unless stated. Before any number enters code, docs or UI copy, the original paper must be read and the figure confirmed.

## 2. Multimedia design and cognitive load

**Coherence, signaling, redundancy.** Mayer's group reports that removing extraneous material, cueing the structure of the content, and avoiding on-screen text that duplicates narration all improve learning, with median effects that are large for coherence and signaling and small for redundancy [S, from a secondary table of Mayer's 2021 meta-analysis]. Redundancy is the least certain: the median is small and the wins are not consistent.

**Split attention.** When learners must read a label in one place and find the matching graphic elsewhere, performance drops. Ginns (2006) and Schroeder and Cenkci (2018) are the two meta-analyses most often cited [S]. Replications have been mixed, and at least one 2019 study questions how robust the effect is [X]. The design rule is still sound: keep the question, the options and the feedback in one place.

**Segmenting.** Breaking a continuous explanation into learner-paced segments helps [S]. This maps directly onto a rehearsal card: one prompt, one decision, then feedback.

**Seductive details.** Interesting but irrelevant material reduces learning. The search did not return an effect size for this principle, so it is recorded here as a design rule without a number.

**Implication.** Each screen should hold one learning task. Decorative or competing elements compete with the task for working memory. This matters more than colour or typography for the learning outcome.

## 3. Retrieval and feedback

**Testing effect.** Testing beats restudy for long-term retention. Meta-analytic estimates range from about g = 0.5 (Rowland, 2014 [S]) to about g = 0.7 (Adesope et al., 2017 [S]). The original Roediger and Karpicke (2006) figures were not confirmed [X].

**Feedback.** Shute's review of formative feedback (Review of Educational Research, 2008) found conflicting results across studies, so its guidelines are a synthesis rather than settled consensus [S/P-metadata]. The guidelines that recur are: feedback should be task-focused, specific and timely, and should tell the learner what to do next rather than judge the learner.

**Implication.** The answer step should be a recall attempt. Feedback should show the correct answer, the reason, and the source, in that order, without a second decision screen. Gamified feedback (points, progress bars) raised motivation without improving learning in one study [S], so it should not replace the learning signal.

## 4. Motion and animation

**Animated versus static.** Höffler and Leutner (2007) found a medium-sized advantage for animation over static pictures across 26 studies and 76 comparisons [S]. Berney and Bétrancourt (2016) found an overall positive effect across 61 studies, with many individual comparisons showing no difference [S]. Castro-Alonso et al. (2019) report a small overall advantage (g = 0.23) [S]. Castro-Alonso, Ayres and Paas (2016) argue that many comparisons are not controlled for moderators, which makes the pooled estimates fragile [S].

**Implication.** Animation helps when it shows a procedure the learner must follow, and does little for decoration. Motion in the UI (transitions, celebratory reveals, the companion character) should be short, optional and off under reduced-motion settings. It should never be on the path to the answer.

## 5. Dashboards and learning analytics

Reviews of learning analytics dashboards (LADs) found that most studies report negligible or small effects on outcomes, and that many compare users with non-users, which confounds the dashboard with engagement [S, Kaliisa et al., arXiv 2312.15042]. Awareness alone may not sustain self-regulation; adaptive dashboards with recommendations may be needed [S].

**Implication.** A progress view is worth building only if it tells the learner what to do next. A grid of numbers without an action is decoration. Any claim that a dashboard improves outcomes needs a controlled comparison inside StudyFrog.

## 6. Motor accessibility and touch

WCAG 2.2 success criterion 2.5.8 (Target Size, Minimum) requires pointer targets of at least 24 by 24 CSS pixels, unless spacing or another exception applies [P]. The W3C rationale is that smaller targets are hard to hit for people with tremor or reduced fine-motor precision, and that the problem is worst where small targets sit close together [P]. Criterion 2.5.5 (AAA) uses 44 by 44 pixels [P]. The benefit extends beyond motor impairment: one-handed use and use on moving transport are cited [P].

**Implication.** StudyFrog's text buttons and rail items should be at least 44 px high on touch-capable layouts, which meets AAA and also helps the planned pen and touch work in EPIC-084.

## 7. Display mode: light versus dark

Evidence is mixed and mostly from small studies [S]. A 2025 *Ergonomics* study reported light mode improved reading performance across groups [S]. The NN/g review reports that light mode helped in daytime, while at night the difference was smaller and participants did not perceive a difference in readability [S]. A 2025 eye-tracking study found lower perceived workload in dark mode [S]. Coding and text-copying studies found little or no effect [S].

**Implication.** Support both modes, default to the system setting, and do not make a claim that either improves learning. Make the contrast of both modes meet the APCA targets already set in the aesthetics note.

## 8. What this means for StudyFrog

1. **One task per screen.** Rehearsal, debrief and planning screens each hold one primary action. Secondary information sits behind a disclosure.
2. **Recall before reveal.** Every item asks for an attempt before showing the answer, and the feedback panel gives the answer, the reason and the source in one place.
3. **Feedback that teaches.** Task-focused, specific, with a next step. No points or streak pressure on the feedback itself.
4. **Motion only where it shows a procedure.** Short, optional, reduced-motion aware.
5. **Progress that leads to action.** A dashboard shows what to study next and why, and any effect claim is tested locally.
6. **Touch-ready targets.** 44 px on touch layouts; keyboard shortcuts stay as the fast path, not the only path.
7. **Both display modes, honest claims.** No evidence-backed claim that one mode improves learning.

## 9. Open questions

- What is the effect of the single-task rehearsal layout on recall, compared with the current layout? This needs a local comparison (see the strategy, section 7).
- Do the testing and feedback effects hold for self-directed adult study, as opposed to classroom use?
- Which tablet and pen behaviours must work first (EPIC-084)?

## 10. Sources

Standards and primary: W3C, *Understanding WCAG 2.2, Success Criterion 2.5.8 Target Size (Minimum)*, `w3.org/WAI/WCAG22/Understanding/target-size-minimum`.

Secondary (as returned by search, not checked against the originals):
- Mayer, R. E. (2021) multimedia principles meta-analysis, via a 2023 secondary table.
- Ginns, P. (2006) spatial contiguity meta-analysis; Schroeder, N. L., & Cenkci, A. T. (2018), *Educational Psychology Review*.
- Chandler, P., & Sweller, J. (1992), the split-attention effect.
- Shute, V. J. (2008), Focus on formative feedback, *Review of Educational Research* 78(1).
- Roediger, H. L., & Karpicke, J. D. (2006), *Psychological Science* 17(3); Rowland, C. A. (2014); Adesope, O. O., et al. (2017).
- Höffler, T. N., & Leutner, D. (2007); Berney, S., & Bétrancourt, M. (2016); Castro-Alonso, J. C., et al. (2019); Castro-Alonso, J. C., Ayres, P., & Paas, F. (2016), "Comparing apples and oranges?"
- Kaliisa, R., et al., arXiv:2312.15042 (learning analytics dashboards review).
- NN/g, "Dark mode" article; a 2025 *Ergonomics* study on display polarity as reported in search results.
