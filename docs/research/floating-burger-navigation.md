# Research & Ergonomic Evaluation: Floating Burger Navigation Menu

**Issue:** [STORY-0147](../issues/stories/STORY-0147-research-floating-burger-navigation-menu.md) / [EPIC-047](../issues/epics/EPIC-047-brand-mascot-icon-and-consolidated-navigation-ux.md)  
**Author:** StudyFrog Architecture & UX Research  
**Date:** 2026-11-10  
**Status:** Completed  

---

## 1. Executive Summary & Problem Space

As StudyFrog expands with advanced academic capabilities—including the **Timetable Schedule Planner**, **Prompt Library**, **Python Console & ScriptRunner**, **Exam Mode**, **Diary**, and **Active Recall Formats**—the application's navigation surface has grown significantly. 

Historically, StudyFrog utilized a static 2-tier navigation structure:
1. **Top Navigation Bar:** Branding mascot (*Breviceps macrops*), breadcrumbs/vault title, profile/vault switchers, search shortcut, and window frame actions.
2. **Left Sidebar:** A vertical stack of up to 14 navigation items (Home, Library, Folders, Rehearse, Calendar, Board, Reports, Prompts, Diary, Achievements, Agent, PDF, Console, Settings).

While this static arrangement is familiar, it presents ergonomic and spatial challenges:
- **Horizontal Screen Real Estate:** The static 180–220 px sidebar consumes ~15–20% of viewport width on typical laptop screens (1366x768 to 1920x1080), compressing reading measure widths below optimal typographic targets (60–75 characters per line).
- **Distraction & Cognitive Clutter:** Persistent visible icons and labels compete with deep-work active recall during card rehearsal and reading.
- **Redundancy with Zen Mode:** Zen mode (`F11`) already hides the sidebar; however, returning to navigation requires an explicit toggle, breaking flow.

This research investigates the ergonomic and architectural viability of replacing or augmenting the static sidebar with a **floating corner burger menu / Floating Action Button (FAB)**.

---

## 2. Spatial Footprint & Typographic Measure Analysis

### 2.1 Viewport Allocation Comparison

| Layout Model | Navigation Footprint | Content Available Width (1440px window) | Reading Measure Compliance (60–75 cpl) |
|---|---|---|---|
| **Static Sidebar + Top Bar** | ~220 px width + 40 px height (~38,000 px²) | ~1220 px | Requires artificial margin clamping |
| **Zen Mode (No Chrome)** | 0 px | 1440 px | Optimal reading columns, but zero instant nav |
| **Floating Burger Menu (FAB)** | 44×44 px button in corner (~1,936 px²) | 1440 px (95% area gain vs static) | Preserves centered 65 cpl typographic measure |

By moving navigation items into a floating drawer or popover, the main central panel gains up to **220 px of horizontal space**, allowing note markdown columns, code blocks, and multi-column calendar grids (such as the 7-day weekly schedule) to breathe without horizontal scrolling.

---

## 3. Fitts's Law & Motor Ergonomics

According to **Fitts's Law** ($MT = a + b \log_2(2D / W)$), the movement time $MT$ to acquire a target is a logarithmic function of distance $D$ and target width $W$:
1. **Screen Corners as "Infinite Targets":** A floating button positioned flush against a screen corner (e.g., bottom-left or top-left) enjoys effective infinite target depth because mouse cursor overshoot is physically bounded by the display edges.
2. **Reduced Visual Search Distance:** Instead of scanning 14 vertically stacked text labels, opening a consolidated radial or grid popover presents destinations within a compact radius ($R \le 120\text{ px}$), minimizing saccadic eye movement and motor displacement.
3. **Touch and Tablet Affordance:** On touch displays, a floating corner target aligns with thumb natural reach zones (especially bottom-left or bottom-right).

---

## 4. Accessibility, Keyboard Flow & Non-Interference Guardrails

To prevent usability regression, the floating burger navigation must adhere to strict UX constraints:

1. **Non-Interference with Study Content:**
   - The button must float above background layers with an elevated surface token (`pal.surface`), subtle shadow/border, and high APCA contrast.
   - It must never overlap primary action triggers (e.g., "Show Answer", "Again", "Good", "Easy" buttons during rehearsal).
   - Recommended position: **Bottom-left corner** (`x: 16px, y: height - 56px`), safely away from center-aligned study cards and bottom-right status indicators.

2. **Keyboard First-Class Citizenship:**
   - `Alt + M` or `Cmd/Ctrl + B`: Instantly toggles the navigation menu drawer/popover.
   - `Escape`: Instantly dismisses the open navigation drawer, returning focus to the active document or study card.
   - Arrow keys (`Up`/`Down` or `Left`/`Right`): Linear traversal across screens with `Enter` confirmation.
   - Global Quick Search / Command Palette (`Ctrl/Cmd + K`): Remains available as a parallel rapid switcher.

3. **Screen Readers & AccessKit Tree:**
   - The trigger must expose `Role::Button` with label `"Navigation menu"`, `expanded` state attribute, and keyboard shortcut hint.
   - The popover list must expose `Role::Menu` or `Role::List` with accessible screen names.

---

## 5. Architectural Recommendation

1. **Hybrid Approach (Best of Both Worlds):**
   - Provide an application setting in **Settings > Appearance**:
     - `Navigation Style: [ Static Sidebar | Floating Burger Button | Auto (Compact on < 1100px) ]`
   - In **Floating Burger Button** mode:
     - The static sidebar is collapsed.
     - A polished floating button featuring the stylized Breviceps macrops glyph or sleek 3-line burger icon rests in the bottom-left corner.
     - Clicking the burger opens an elevated navigation drawer with search filter, screen categories (Study, Planning, Knowledge, Automation, System), and recent history breadcrumbs.
   - In **Zen Focus Mode** (`F11`):
     - The floating button automatically dims to 20% opacity until hovered, delivering true distraction-free immersion.

---

## 6. Implementation Prototype

The prototype is implemented in `crates/studyfrog-ui/src/view/burger_menu.rs`, providing:
- Floating corner trigger with subtle hover elevation
- Dropdown/drawer listing all application screens with keyboard shortcut hints
- Smooth integration with `State::screen` and `Msg::Nav`
- 100% theme palette compliance (zero hardcoded colors) and $\le 250$ lines of pure Rust code.
