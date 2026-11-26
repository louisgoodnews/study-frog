# StudyFrog 1.4.0

Exam planning and adaptive authoring release.

## Added

- **Exam Scheduler with adaptive preparation window (`EPIC-076`):**
  - Link stacks and items to upcoming exams (`ExamPlan` / `ExamLink`) with local SQLite persistence.
  - Adaptive preparation-window estimator and windowed item scheduler that distributes and rebalances exam-linked items across a user-set or stats-proposed week window.
  - Exam Scheduler modal with live workload preview, plus Calendar, Home, and Rehearsal integration for scheduled exam blocks.
- **Personalized WIP item-complexity warnings (`EPIC-077`, `STORY-0252`):**
  - Computes a personal retention-complexity profile from reviewed items in the active vault.
  - Displays a live, dismissible warning banner in the **Create an item** modal when a draft Card or Question becomes significantly more complex than the user's retained baseline.

## Install

Download `StudyFrog-1.4.0-x86_64.AppImage` (or `StudyFrog-1.4.0-x86_64.tar.gz`) from this release, verify it against the `.sha256` file printed next to it, and launch it from a desktop session.

## Upgrading from 1.3.0

Drop-in upgrade: existing vaults, settings, and databases open without manual migration.
