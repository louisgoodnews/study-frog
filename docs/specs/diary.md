# Diary

The diary allows learners to record self-reported reflections, ratings, and context alongside their measured study results (STORY-0068, EPIC-024). It brings metacognitive insight and calibration to Study Frog: learners can compare how a session *felt* against objective accuracy and familiarity, discovering whether they tend to over- or underrate their grasp.

Diary entries are stored in the vault as human-readable Markdown files. Like all content in Study Frog, diary entries remain local, private, and editable with standard text tools. Diary ratings are strictly observational: they **never** alter SRS intervals, card schedules, or memory states.

---

## 1. File format and storage

### Location and naming
- **Vault folder:** Diary entries live in the `diary/` directory at the vault root (for example `vault/diary/2026-11-04.md`). Folders named `_diary/` are also recognized for compatibility.
- **Filename convention:**
  - Standard single daily entry: `diary/YYYY-MM-DD.md` (e.g. `diary/2026-11-04.md`).
  - Multiple entries on the same day: suffixed with a hyphenated counter or timestamp, for example `diary/YYYY-MM-DD-1.md`, `diary/YYYY-MM-DD-2.md`, or `diary/YYYY-MM-DD-1430.md`.
- **Kind:** Identified by front-matter `kind: diary`.
- **Single source of truth:** The Markdown file is authoritative. The SQLite index indexes entries and ratings for fast queries, filtering, and reports, but the database can be rebuilt from the vault files at any time.

### Front-matter schema
Every diary entry begins with YAML front-matter:

| Key | Type | Required | Meaning |
|---|---|---|---|
| `id` | ULID string | Yes | Unique stable identifier (26 characters, canonical ULID). Never reused. |
| `kind` | string | Yes | Must be `diary`. |
| `date` | ISO date string (`YYYY-MM-DD`) | Yes | Civil date of the diary entry. |
| `title` | string | No | Optional entry heading; defaults to civil date display (e.g. "Diary: 2026-11-04"). |
| `created` | RFC 3339 datetime | No | When the entry was first created (UTC). |
| `updated` | RFC 3339 datetime | No | When the entry was last edited (UTC). |
| `study_minutes` | integer (>= 0) | No | Total self-reported or run-tracked study time in minutes. |
| `tags` | list of string | No | Free-form tags (e.g. `[morning, exam-prep, library]`). |
| Standard rating keys | integer (1..=5) | No | Optional standard ratings (see Section 2). |
| Custom rating keys | integer / string | No | User-defined ratings defined in settings (see Section 3). |
| `runs` | list of ULID strings | No | Rehearsal runs associated with this entry. |
| `stacks` | list of ULID strings or folder paths | No | Stacks studied or referenced. |
| `items` | list of ULID strings | No | Specific cards, notes, or questions referenced. |
| `exams` | list of strings | No | Calendar exam or session IDs referenced. |

Unknown front-matter keys are preserved verbatim across round trips.

### Markdown body
- **Reflections:** Free Markdown text containing personal reflections, takeaways, obstacles encountered, and qualitative observations.
- **Wikilinks:** Standard wikilink syntax (`[[Title]]`, `[[Title|Label]]`, `[[Title#Heading]]`) can be used to link to any note, card, question, or stack. Links are indexed into the vault link graph.
- **Pictures:** Images follow the standard vault picture schema (`![Description](assets/name.png)`) governed by STORY-0044. Pictures are stored in the `assets/` folder and backed up with the vault.

### Example diary file
```markdown
---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
title: Evening Biochemistry Rehearsal
created: 2026-11-04T19:45:00Z
updated: 2026-11-04T19:52:30Z
study_minutes: 35
tags: [biochem, enzymes, quiet-evening]
performance: 4
success: 4
confidence: 3
effort: 4
focus: 4
energy: 3
difficulty: 4
satisfaction: 4
understanding: 4
mood: 4
stress: 2
motivation: 4
sleep: 3
runs: [01K2V8X3Y4Z5A6B7C8D9E01234]
stacks: [biochem/metabolism]
items: [01K2P4Q5R6S7T8U9V0W1X2Y3Z4]
---
Felt in the zone tonight after dinner. The Michaelis-Menten kinetics cards from [[Enzyme Kinetics]] were tough at first, but deriving the Lineweaver-Burk equation finally clicked.

![Lineweaver-Burk plot](assets/enzymes-plot-01k2v9.png)

Need to review Km vs Vmax inhibitors tomorrow before the mock exam.
```

---

## 2. Rating scales and wording

Study Frog defines 13 standard self-assessment scales. Each scale uses a discrete 5-point rating (integers **1 to 5**), providing a clear midpoint (3) and well-defined polar endpoints (1 and 5).

Every rating is **strictly optional**. Learners may answer one, several, or none. Settings allows learners to select which scales are active in the entry form.

### Standard scales overview

| Key | Dimension | Short prompt |
|---|---|---|
| `performance` | Performance | How well did you perform overall? |
| `success` | Goal success | Did you achieve your intended study goal? |
| `confidence` | Confidence | How certain are you in what you know? |
| `effort` | Mental effort | How hard did you have to work mentally? |
| `focus` | Focus & Attention | How focused and free of distraction were you? |
| `energy` | Energy level | What was your physical and mental energy level? |
| `difficulty` | Perceived difficulty | How difficult was the material or session? |
| `satisfaction` | Satisfaction & Retention | How satisfied are you with your progress and retention? |
| `understanding` | Conceptual understanding | How deeply did you understand the core concepts? |
| `mood` | Mood | How did you feel emotionally during study? |
| `stress` | Stress & Pressure | How much stress or pressure did you feel? |
| `motivation` | Motivation | How strong was your drive and desire to study? |
| `sleep` | Restedness & Sleep | How rested were you from your previous sleep? |

### Exact wording for each scale level (1 to 5)

The owner accepted the wording of the standard scales on 2026-11-04:

#### 1. Performance (`performance`)
*Question: How well did you perform overall during this session?*
- **1 – Poor:** Struggled significantly; remembered very little and made frequent errors.
- **2 – Below average:** Below expectations; hesitant recall with notable gaps.
- **3 – Average:** Acceptable performance; remembered core material with expected slip-ups.
- **4 – Good:** Strong performance; prompt and accurate responses across most material.
- **5 – Outstanding:** Flawless recall; effortless mastery and immediate recall throughout.

#### 2. Goal success (`success`)
*Question: Did you achieve your intended study goal for this session?*
- **1 – Not achieved:** Abandoned early or failed to complete intended targets.
- **2 – Partially achieved:** Completed less than half of what was planned.
- **3 – Mostly achieved:** Finished the core task, leaving minor items unfinished.
- **4 – Fully achieved:** Completed all planned items and reviews as intended.
- **5 – Exceeded:** Surpassed the study goal, completing extra material or deep reviews.

#### 3. Confidence (`confidence`)
*Question: How certain and confident are you in your grasp of the material?*
- **1 – Very doubtful:** Constantly guessing; low belief in retention.
- **2 – Insecure:** Unsteady grasp; frequent second-guessing.
- **3 – Moderately confident:** Reasonable confidence in familiar areas, uncertain on edges.
- **4 – Confident:** Solid assurance; sure of answers without hesitation.
- **5 – Fully confident:** Complete conviction; total trust in retention and mastery.

#### 4. Mental effort (`effort`)
*Question: How much mental exertion was required?*
- **1 – Minimal effort:** Breezed through without active strain.
- **2 – Light effort:** Modest concentration required; smooth pacing.
- **3 – Moderate effort:** Balanced mental exertion; healthy working engagement.
- **4 – High effort:** Intensive cognitive strain; sustained hard concentration.
- **5 – Exhausting effort:** Maximum cognitive exertion; pushed mental capacity to the limit.

#### 5. Focus & Attention (`focus`)
*Question: How well did you maintain concentration and resist distractions?*
- **1 – Highly distracted:** Constant interruptions, daydreaming, or multitasking.
- **2 – Wandering:** Frequent lapses in focus; difficult to stay on track.
- **3 – Moderate focus:** Mostly on task with periodic brief distractions.
- **4 – Sharp focus:** Sustained attention; easily ignored minor distractions.
- **5 – Deep immersion:** Total immersion / flow state; completely undivided attention.

#### 6. Energy level (`energy`)
*Question: What was your energy and vitality level during the session?*
- **1 – Drained:** Exhausted and fatigued; struggling to stay awake.
- **2 – Low:** Sluggish and lethargic; low physical/mental stamina.
- **3 – Steady:** Normal baseline energy; sufficient to study comfortably.
- **4 – Energetic:** Lively and alert; active mental presence.
- **5 – Peaked:** High vigor and peak alertness; brimming with vitality.

#### 7. Perceived difficulty (`difficulty`)
*Question: How challenging did the session content feel?*
- **1 – Very easy:** Trivial; basic recall requiring no problem-solving.
- **2 – Easy:** Straightforward; comfortable material with few obstacles.
- **3 – Moderate:** Balanced; appropriate level of challenge and friction.
- **4 – Hard:** Demanding; complex concepts requiring deliberate analysis.
- **5 – Very hard:** Extremely difficult; conceptually dense or unfamiliar obstacles.

#### 8. Satisfaction & Retention feeling (`satisfaction`)
*Question: How satisfied are you with your session and how well it will stick?*
- **1 – Disappointed:** Frustrated; feeling that time was largely wasted.
- **2 – Unsatisfied:** Discontented; doubt that the material will stick.
- **3 – Content:** Reasonable progress; adequately satisfied with the outcome.
- **4 – Satisfied:** Solid accomplishment; confident the knowledge will stick.
- **5 – Delighted:** Thrilled with progress; deep sense of fulfillment and durable memory.

#### 9. Conceptual understanding (`understanding`)
*Question: How deeply did you comprehend the underlying concepts?*
- **1 – Surface rote:** Pure surface memorization without grasping the underlying logic.
- **2 – Vague grasp:** Foggy comprehension; can recognize patterns but cannot explain why.
- **3 – Working understanding:** Understand the main mechanisms and standard cases.
- **4 – Thorough comprehension:** Clear understanding of concepts and their relations.
- **5 – Deep insight:** Comprehensive mastery; can explain, synthesize, and transfer easily.

#### 10. Mood (`mood`)
*Question: What was your emotional disposition while studying?*
- **1 – Frustrated / Negative:** Irritable, bored, or resentful toward studying.
- **2 – Subdued:** Unenthusiastic or flat; treating study as a chore.
- **3 – Neutral:** Calm and even-keeled; steady emotional baseline.
- **4 – Positive:** Cheerful and receptive; enjoying the process.
- **5 – Joyful / Inspired:** Enthusiastic, curious, and excited by learning.

#### 11. Stress & Pressure (`stress`)
*Question: What degree of stress, pressure, or anxiety did you feel?*
- **1 – Completely serene:** Total peace and relaxation; zero tension.
- **2 – Low stress:** Mild awareness of deadlines, but comfortably calm.
- **3 – Moderate pressure:** Noticeable deadline or test pressure; manageable arousal.
- **4 – High stress:** Pronounced anxiety or worry; uneasy tension.
- **5 – Overwhelmed:** Acute anxiety or panic; paralyzing stress.

#### 12. Motivation (`motivation`)
*Question: How strong was your drive and desire to study?*
- **1 – Resistant:** Severe procrastination; had to force every minute.
- **2 – Reluctant:** Low inclination; studied purely out of obligation.
- **3 – Willing:** Neutral willingness; ready to get the work done.
- **4 – Motivated:** Keen interest and proactive engagement.
- **5 – Passionate:** Intense drive and curiosity; eager to explore and learn.

#### 13. Restedness & Sleep (`sleep`)
*Question: How rested and refreshed did you feel from your prior sleep?*
- **1 – Severely sleep-deprived:** Barely slept or heavy insomnia; cognitive fog.
- **2 – Poorly rested:** Fragmented or brief sleep; woke up tired.
- **3 – Adequately rested:** Sufficient rest; functioning normally.
- **4 – Well-rested:** Good night's sleep; alert and refreshed.
- **5 – Fully rejuvenated:** Ideal, uninterrupted deep sleep; peak physical readiness.

---

## 3. User-defined scales

Learners may configure custom rating scales to track personal variables that affect their learning (such as caffeine consumption, study location, environmental noise, or posture).

### Configuration in TOML
Custom scales are declared in `studyfrog.toml` (vault-scoped) or `app.toml` (profile-scoped):

```toml
[[diary.custom_scale]]
id = "caffeine"
name = "Caffeine intake"
prompt = "How much caffeine did you consume before or during study?"
min = 1
max = 5
labels = [
  "None",
  "Low (1 tea or light coffee)",
  "Moderate (1-2 coffees)",
  "High (3+ coffees or energy drink)",
  "Jittery / Excessive"
]
```

### Scale constraints
- `id`: Non-empty alphanumeric string containing ASCII letters, digits, and underscores (`^[a-z][a-z0-9_]*$`). Must not collide with standard scale IDs.
- `name`: Display title in the form and reports (e.g. "Caffeine intake").
- `prompt`: Explanatory prompt question shown as helper text.
- `min` / `max`: Range bounds (integers, defaults `min = 1`, `max = 5`).
- `labels`: Array of descriptive strings matching the range size (`max - min + 1`). If omitted, raw numbers 1..5 are displayed.

### Storage in front-matter
Custom ratings are stored as top-level keys matching their `id`:
```yaml
caffeine: 2
```
Unknown or unconfigured scale keys found in existing front-matter are preserved without alteration.

### Settings UI
In the Settings screen ("Diary" tab):
- Users can enable or disable each of the 13 standard scales.
- Users can choose up to **3 primary scales** to display in the compact post-run prompt (defaults: `focus`, `energy`, `satisfaction`).
- Users can add, edit, reorder, or delete custom scales.

---

## 4. Bidirectional links

Diary entries connect organically with the rest of the vault, forming bidirectional links between subjective reflections and objective entities.

### Link types and references
1. **Rehearsal runs (`runs: [ULID, ...]`):**
   - Links the diary entry directly to a specific rehearsal run in the database (`run` table).
   - Allows instant comparison between the user's subjective rating and the run's measured latency, accuracy, and pacing.
2. **Stacks (`stacks: [ULID or folder, ...]`):**
   - Links the entry to one or more primary or referenced stacks.
   - Enables filtering diary entries by subject or course.
3. **Items (`items: [ULID, ...]`):**
   - Links specific cards, notes, or questions that caused trouble, inspiration, or required follow-up.
4. **Exams and study sessions (`exams: [ID, ...]`):**
   - Links entries to scheduled calendar events, exams, or milestone sessions.
5. **Inline Wikilinks:**
   - Any `[[WikiLink]]` in the diary body is parsed and linked in the vault graph.

### Bidirectional display in the UI
- **In the Diary entry view:** A header strip displays interactive chips for all linked runs, stacks, and items. Clicking a run chip opens the rehearsal summary review for that run; clicking a stack chip navigates to the stack; clicking an item opens the item viewer.
- **In the Rehearsal run summary / review:** If a diary entry was logged for this run, a "Diary reflection" card is displayed alongside run metrics, showing the ratings and note.
- **In the Stack view:** The stack's details show a "Diary" counter or section listing recent diary entries mentioning or tagged with this stack.
- **In the Item view:** The item's metadata panel lists backlinks from diary entries, allowing learners to see what they previously noted when reviewing this item.

---

## 5. Post-run prompt

Immediately after finishing a rehearsal run (on the run summary screen, TASK-0173), Study Frog presents a quiet, optional prompt allowing learners to log their immediate thoughts while the session is fresh.

### Principles: calm and non-blocking
- **Never modal:** The prompt is embedded directly at the bottom of the run summary screen. It does not pop up a dialog or steal focus abruptly.
- **Single-key dismissal:** Pressing `Space` (Finish run) or `Esc` immediately exits the summary screen without saving an entry. If no ratings or text were input, nothing is written.
- **Zero friction:** Quick ratings can be chosen with numeric keys (`1`..`5`) or mouse clicks.

### Layout and interaction
```
+-------------------------------------------------------------+
| Rehearsal Complete: 24 items in 8m 12s                      |
| Accuracy: 88%  |  Median latency: 2.4s  |  Status: Good     |
+-------------------------------------------------------------+
|                                                             |
|  How did that go? (Optional diary note)                     |
|                                                             |
|  Focus:        [1] [2] [3] (4) [5]   Deep immersion         |
|  Energy:       [1] [2] (3) [4] [5]   Steady                 |
|  Satisfaction: [1] [2] [3] (4) [5]   Satisfied              |
|                                                             |
|  Reflection: [ Deriving Lineweaver-Burk clicked tonight... ]|
|                                                             |
|  [ Save entry (Enter) ]   [ More details... ]   [ Skip (Esc)]|
+-------------------------------------------------------------+
```

1. **Compact prompt bar:**
   - Displays the user's selected primary scales (defaults: `focus`, `energy`, `satisfaction`).
   - A clean one-line text field for quick notes.
   - Hovering or selecting any level displays its descriptive wording (e.g. "4 – Deep immersion").
2. **Auto-populated metadata:**
   - `date`: Current local civil date.
   - `study_minutes`: Rounded active monotonic rehearsal time.
   - `runs`: Current run ULID.
   - `stacks`: Stacks rehearsed in this run.
3. **Actions:**
   - **Enter / Save entry:** Writes the entry file `diary/YYYY-MM-DD.md` (or adds a suffix if an entry exists), indexes it, and returns to the library/home.
   - **More details...:** Expands the full diary editor (showing all active scales, tag input, and full Markdown body editor).
   - **Skip / Esc / Space:** Closes the summary without writing a diary file.

---

## 6. Diary screen and Home integration

### Diary screen (TASK-0172)
The Diary screen provides a comprehensive view of historical self-assessments:
- **Calendar & Month view:** Highlights days with entries; color-coded or dotted by average satisfaction or focus.
- **Chronological entry list:** Browse past entries with quick search and filters:
  - Filter by date range.
  - Filter by stack or subject.
  - Filter by rating thresholds (e.g. `stress >= 4` or `difficulty >= 4`).
- **Inline Markdown editor:** Full distraction-free editor with syntax highlighting, wikilink auto-completion, image drop support, and rating controls.

### Home screen quiet nudge
- When no diary entry has been made yet for the current civil day, the Home screen displays an unobtrusive hint: *"No diary entry yet today"*, with a quick "Write entry" button.
- In keeping with the calm UI philosophy, this hint is subtle, non-intrusive, and can be disabled in Settings (`show_diary_hint = false`).

---

## 7. Metacognitive calibration reports

TASK-0174 introduces the self-assessment calibration report (`studyfrog-report`):

- **Subjective vs. Objective comparison:**
  - Compares self-rated `performance`, `confidence`, and `understanding` against objective test metrics: measured accuracy (% recalled without Again), FSRS retrievability, and familiarity score.
- **Calibration bias curve:**
  - Identifies systematic cognitive biases:
    - **Over-confidence bias:** High self-ratings coupled with lower measured accuracy or high lapse rates.
    - **Under-confidence bias:** Low self-ratings despite strong objective recall and fast latency.
    - **Accurate calibration:** Self-ratings strongly correlate with objective outcomes.
- **Sample size indicator:**
  - All calibration charts explicitly state the underlying evidence (for example: *"Based on 18 diary entries across 420 reviews over the last 30 days"*).
- **Independence:**
  - Diary ratings serve self-knowledge and metacognitive reflection only. They are never fed into SRS scheduling weights or interval calculations.
