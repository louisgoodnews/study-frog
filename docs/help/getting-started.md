# Getting started

This page takes you from an empty folder to your first rehearsal and a planned study session. It takes about ten
minutes. The "First steps" tutorial walks you through the same steps inside the app (see [[help|Using help]]).

## 1. Open or create a vault

A **vault** is a folder of Markdown files that StudyFrog reads and writes.

1. On the welcome page press **Open or create a vault…**.
2. Press **Choose folder…** and pick a folder. Choose an existing folder of notes, or an empty folder to start a
   new vault.
3. Press **Open vault**. StudyFrog indexes the folder and opens the [[home|Home]] screen.

StudyFrog remembers the vault. Next time it opens straight away. The top bar shows the vault's folder name next to
StudyFrog; hover over it to see the full path. To switch vaults, use the vault switcher at the left of the top bar,
or **Close vault** in the profile menu.

If you have a backup, press **Restore from a backup…** on the welcome page instead. See [[settings]].

## Finding your way around

- **The top bar**, from left to right: **Show or hide the sidebar**, the vault switcher, StudyFrog and the open
  vault's name, **Search items** (finds items and PDFs by title and text; **Enter** opens the first result), **+
  Create**, the bell with your [[notifications]], **?** for [[help|help]], the gear for [[settings]], and the person
  button for profiles and **Close vault**.
- **The sidebar** lists the screens: Home, Library, Calendar, Board, Achievements, Reports, Assistant, Graph,
  Rehearse and Settings.
- **The status bar** at the bottom shows the latest message, or the size of your vault.

## 2. Create a stack

Every card, note and question lives in a **stack** (like a deck or a notebook). A stack is a folder in your vault.

1. Press **+ Create** in the top bar.
2. Choose **Stack** as the type in the top left corner.
3. Give it a title, for example "French". You can add a description and a due date.
4. Press **Create**.

Stacks can sit inside other stacks: choose a parent stack in the top left corner. See [[create-dialog]].

## 3. Create a card

1. Press **+ Create** again and choose **Card**.
2. Pick the stack you just made.
3. Write the **Front** (the question) and the **Back** (the answer). Markdown works in both.
4. Press **Create**. Tick **Create another** first if you want to make several cards in a row.

You can also make **notes** (free text) and **questions** (answered by typing). See [[item-page]] to edit an item
later.

## 4. Rehearse

1. Choose **Rehearse** in the sidebar, or press **Start today's rehearsal** on Home.
2. Step 1, **What**: tick your stack (or leave everything unticked for all stacks).
3. Step 2, **How**: keep the preset **Spaced repetition** for now.
4. Step 3, **Review**: check your choices and press **Start rehearsal**.
5. Press **Space** to see the answer, then **1**, **2**, **3** or **4** to rate it Again, Hard, Good or Easy.

After the run a review shows every item with your rating. See [[rehearse]] and [[keyboard]].

## 5. Plan a study session

1. Choose **Calendar** in the sidebar.
2. Click a day and press **Add to calendar**.
3. Choose the kind **study**, give it a title and a time, and link it to your stack.
4. Press **Save entry**.

The session shows on Home under **Today** on that day, and you can start a rehearsal for its stack from the
calendar. See [[calendar]].

## Where your things are

- Your cards, notes and questions are `.md` files in the stack folders.
- Pictures you add go into the vault's `assets/` folder.
- Vault-wide settings are in `studyfrog.toml`, a stack's settings in its `_stack.toml`. See [[settings]].
- Reviews, runs, the board and the calendar are kept in a database inside the vault's hidden `.studyfrog` folder.
  Make a backup now and then (Settings, **Data** tab).

## Next steps

- Learn the screens: [[home]], [[library]], [[stack-page]], [[reports]].
- Read [[why-studyfrog|Why StudyFrog]] for the ideas behind the app.
- Stuck? Try the [[faq]] or the [[glossary]].

## Leaving StudyFrog
The red **Exit** button at the right end of the top bar closes the app. If you are editing a document with unsaved
changes, it asks first.
