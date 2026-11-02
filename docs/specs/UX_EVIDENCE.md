# UX evidence and the rules we take from it

What the research says, how strong it is, and which rule in StudyFrog it backs. Strength is honest: where
the evidence is practitioner lore or contested, it says so, and we treat the rule as a default to test
with users, not a law. Sources were checked by search on 2026-10-30; effect sizes quoted are from the
abstracts or secondary summaries and should be checked against the papers before being relied on.

## Findings

| Finding | Strength | Source |
|---|---|---|
| Working memory holds about 4 chunks, not Miller's 7 plus or minus 2 | Strong, debated in detail | Cowan 2001, *Behavioral and Brain Sciences* 24(1) |
| Reducing extraneous cognitive load improves learning and task performance | Strong (theory with many experiments) | Sweller 1988, cognitive load theory |
| Choice reaction time grows with the number of options (Hick-Hyman) | Strong for lab tasks; weaker for menus users know | Hick 1952, Hyman 1953 |
| Practice testing and distributed practice are the two high-utility study techniques; rereading and highlighting are low | Strong (review of ten techniques) | Dunlosky et al. 2013, *Psychological Science in the Public Interest* 14(1) |
| Specific if-then plans ("when X, I do Y") raise goal completion: 94 tests, d about 0.65 (secondary figure); a 2025 review covers 642 tests | Strong for discrete goals, smaller for sustained habits | Gollwitzer and Sheeran 2006; Sheeran, Listrom, Gollwitzer 2025 |
| Plan reminders improve compliance and recall of the plan, but may create dependence on the reminder | Weak (one 4-week study) | Wicaksono et al., in *Interacting with Computers* |
| Constant-speed progress bars do not reduce drop-off on average; fast-to-slow ones do, slow-to-fast ones raise it | Moderate (32 randomized web-survey experiments; the coding has been disputed) | Villar, Callegaro, Yang 2013, *Social Science Computer Review* 31(6) |
| Visible progress toward a goal raises the chance of finishing it (endowed progress) | Moderate | Nunes and Dreze 2006 |
| Attractive interfaces are rated more usable; the effect can reverse when real usability differences are strong | Contested (one study, one failed reversal) | Tractinsky, Katz, Ikar 2000; Tuch et al. 2012 |
| Multi-step forms beat single-page forms | **No credible evidence.** Vendor blogs conflict; one large test found equal net conversion | none peer reviewed |
| Progressive disclosure helps | Practitioner consensus; its inventors found little experimental support | Carroll and Rosson 1987 |
| Web pages are read in an F-shaped pattern | Disputed; not shown for dashboards | NN/g 2006 vs. EyeQuant analysis |

## Rules we adopt

1. **Few chunks per screen (Cowan, Sweller).** A step or panel carries about four groups of controls at most.
   The rehearsal setup is split into three short steps for this reason, not because wizards are proven
   better. We keep a "jump to any step" control so power users lose nothing.
2. **One obvious primary action per view (Hick).** One filled button; everything else is plain.
3. **Show what the learning science says works (Dunlosky).** The home page leads with what is due (spaced
   practice) and offers testing (rehearsal) as the main action; nothing nudges users toward rereading.
4. **Plans with a time and a trigger (Gollwitzer).** Study sessions, tasks, exams and stack deadlines go on the
   calendar with a date, a time and one or more reminders. Reminders are shown once and then left alone, so
   they do not become a crutch.
5. **Progress that starts fast (Villar, Nunes).** The run progress bar counts items finished and the setup
   shows its step as "step 2 of 3". We do not promise a total that is wrong: skipped and repeated items change
   it, and the bar says so.
6. **Fill the space with information, not decoration.** The home page and the calendar use the whole window,
   showing due items and upcoming events instead of empty areas. No F-pattern is assumed; the most important
   block (what is due today) sits top left because that is where the eye starts in left-to-right layouts.
7. **Looks matter a little, usability matters more (Tractinsky vs. Tuch).** The shared theme exists for
   consistency, but we judge screens by task success, not by looks.

## Decisions reversed

- **The vault name in the top bar (2026-11-02, TASK-0128, reversing STORY-0041).** STORY-0041 removed the vault folder name
  from the bar, the status line and notices. The owner asked for it back in the bar because several vaults and profiles can
  be used and the bar did not say which one was open (visibility of system status, Nielsen). The bar shows the folder name
  only, cut at 24 characters with an ellipsis; the path is only in the tooltip. The status line and notices stay as
  STORY-0041 left them.

## What we do not know

We have no data from StudyFrog users yet. The release checklist should include a five-person task test (set
up a run, plan an exam, find a due card) before any claim that the new flow is better.
