# Learning science

Findings that StudyFrog's design relies on.

Verification status (checked 2026-10-30 against PubMed/publisher listings): **[v]** = bibliographic
details confirmed, **[u]** = from memory, unverified. Check findings (not just citations) before
quoting them in user-facing text.

| Principle | Design consequence | Source |
|---|---|---|
| **Spacing effect**: distributed practice beats massed practice, with optimal gaps growing with retention interval. | SRS scheduling (FSRS), forecasts of review load. | Cepeda, Pashler, Vul, Wixted, Rohrer (2006), "Distributed practice in verbal recall tasks", *Psychological Bulletin* 132(3), 354-380, doi:10.1037/0033-2909.132.3.354 **[v]**. Ebbinghaus (1885), *Über das Gedächtnis* **[u]**. |
| **Testing effect / retrieval practice**: retrieving beats re-reading. | Rehearsal runs ask before showing; free-text answering. | Roediger & Karpicke (2006), "Test-enhanced learning", *Psychological Science* 17(3), 249-255 **[v]**. Karpicke & Blunt (2011), "Retrieval practice produces more learning than elaborative studying with concept mapping", *Science* 331(6018), 772-775, doi:10.1126/science.1199327 **[v]**. |
| **Desirable difficulties**: effortful retrieval improves long-term retention. | Target retention below 100%; interleaving stacks. | Bjork (1994), in *Metacognition*, MIT Press **[u]**. |
| **Practice technique utility**: practice testing and distributed practice rated high utility. | Prioritise those modes in the product. | Dunlosky et al. (2013), *Psychological Science in the Public Interest* 14(1), 4-58, doi:10.1177/1529100612453266 **[v]**. |
| **Forgetting curve modelling** | Retrievability and forecasts. | open-spaced-repetition FSRS project (github.com/open-spaced-repetition), papers listed there **[u]**. |
| **Good card design** (minimum information, one fact per card). | Authoring guidance, card-quality hints. | Wozniak (1999), "Effective learning: Twenty rules of formulating knowledge" **[u]**. |
| **Latency as a memory signal**: faster correct retrieval indicates stronger memory. | Capture time-to-flip/answer; report distributions. | Hypothesis to validate in M7; see retrieval-fluency literature. |

## Open questions
Whether latency improves scheduling accuracy beyond ratings is unproven here; M7 reports will
test it on collected data before any scheduler change.
