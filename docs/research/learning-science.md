# Learning science

Findings that StudyFrog's design relies on. Verify citations before quoting in user-facing text.

| Principle | Design consequence | Source |
|---|---|---|
| **Spacing effect**: distributed practice beats massed practice, with optimal gaps growing with retention interval. | SRS scheduling (FSRS), forecasts of review load. | Cepeda, Pashler, Vul, Wixted, Rohrer (2006), "Distributed practice in verbal recall tasks", *Psychological Bulletin* 132(3). Ebbinghaus (1885), *Über das Gedächtnis*. |
| **Testing effect / retrieval practice**: retrieving beats re-reading. | Rehearsal runs ask before showing; free-text answering. | Roediger & Karpicke (2006), "Test-enhanced learning", *Psychological Science* 17(3). Karpicke & Blunt (2011), *Science* 331. |
| **Desirable difficulties**: effortful retrieval improves long-term retention. | Target retention below 100%; interleaving stacks. | Bjork (1994), in *Metacognition*, MIT Press. |
| **Practice technique utility**: practice testing and distributed practice rated high utility. | Prioritise those modes in the product. | Dunlosky et al. (2013), *Psychological Science in the Public Interest* 14(1). |
| **Forgetting curve modelling** | Retrievability and forecasts. | open-spaced-repetition FSRS project (github.com/open-spaced-repetition), papers listed there. |
| **Good card design** (minimum information, one fact per card). | Authoring guidance, card-quality hints. | Wozniak (1999), "Effective learning: Twenty rules of formulating knowledge". |
| **Latency as a memory signal**: faster correct retrieval indicates stronger memory. | Capture time-to-flip/answer; report distributions. | Hypothesis to validate in M7; see retrieval-fluency literature. |

## Open questions
Whether latency improves scheduling accuracy beyond ratings is unproven here; M7 reports will
test it on collected data before any scheduler change.
