# Frequently asked questions

Short answers to common questions, and honest notes on what StudyFrog cannot do yet. If your question is not here,
try **Search help** or the [[glossary]].

## Getting started

### What do I need to run StudyFrog?

Linux with Wayland or X11 and OpenGL. The folder and file dialogs go through your desktop's file chooser (the XDG
desktop portal). StudyFrog comes as an AppImage (make it executable and run it) or as a `.tar.gz` with the program,
a desktop entry and an icon. See [[getting-started|Getting started]].

### Is there a Windows, Mac or phone version?

No. StudyFrog is for Linux for now.

### Can I use my existing folder of Markdown notes?

Yes. Open the folder as a vault. Folders that hold a `_stack.md` file are stacks. Files StudyFrog cannot read are
listed under **Problems** in the library's **Overview**, with the reason; they are never changed.

### Can I have several vaults?

Yes. Use the vault switcher at the left of the top bar to open a recent vault or **Open another vault…**. The top bar
shows the name of the open vault; hover over it for the full path. Each vault has its own history, board, calendar
and notifications.

### What are profiles for?

A profile keeps its own settings, recent vaults and API keys, for example one for school and one for work. Use the
person button at the right of the top bar. See [[settings]].

## Your data

### Where is my data, and is it private?

Your content is plain Markdown in your vault folder. Your history is in a database in the vault's hidden
`.studyfrog` folder. Nothing leaves your computer unless you connect a remote AI model and allow it. See
[[privacy]].

### Is there cloud sync?

No, and there is no account. If you want your vault on several computers, sync the folder yourself with a tool you
trust. Do not open the same vault on two computers at the same time.

### Can I edit the files with another editor?

Yes, that is the point. StudyFrog notices changes within a second or two. If you edit a file in another editor while
it is open in StudyFrog, StudyFrog will not overwrite your other edit when you save: it says the file changed, and
you can use **Reload from disk** or **Keep mine**. See [[item-page]].

### How do I make a backup?

Settings, **Data** tab, **Back up now…**. A backup is one `.tar` file. Restoring always goes into a new or empty
folder. See [[settings]].

### How do I move my cards to or from Anki or Obsidian?

Anki packages (`.apkg`) and Obsidian vaults can be imported, with a preview first. StudyFrog exports to JSON, CSV or
Markdown, not to Anki's format; many tools, Anki included, can import CSV. See [[import-export]].

### StudyFrog closed in the middle of a run. Is my work lost?

No. Every item is saved as you go. The unfinished run is saved as stopped the next time you open the vault.

## Rehearsing

### Why is an item not in my rehearsal?

A spaced run shows only items that are due, up to your stacks' daily limits. An item is also left out when it or a
stack above it is **Complete** or **On hold**, when its stack's due date has passed, or when its familiarity is at or
above `familiarity_threshold`. To go through everything regardless, choose another order, such as **Everything, in
order**. See [[rehearse]] and [[statuses]].

### Why did my ratings not change the schedule?

The run was practice only. All orders except **Due first** are practice only by default. Tick **Update my review
schedule with these ratings** in the run setup if you want them to count.

### Do my easy, medium and difficult marks change the schedule?

Not by themselves. They feed rating suggestions, reports and [[familiarity]]. Your rating changes the schedule.

### Can I change a rating after the run?

Yes, in a run that updates the schedule: the review after the run may suggest a different rating, and you can apply
it. The item's schedule is rebuilt as if you had pressed that rating.

### StudyFrog said my wrong typed answer was correct. Why?

The comparison measures **similarity, not truth**. An answer that differs from the reference by one decisive word
("1889" instead of "1789") looks almost the same and usually scores as correct. Treat the match as a hint and rate
honestly yourself. For questions, a `rubric` (a list of key points) in the item's front matter helps.

### How do I get better answer comparison?

Install the small language model: put the files `config.json`, `tokenizer.json` and `model.safetensors` of
`all-MiniLM-L6-v2` (from sentence-transformers) into `~/.local/share/studyfrog/models/all-MiniLM-L6-v2/`. StudyFrog
uses it by itself and says which method compared each answer. Without it, word matching is used. It runs on your
computer; nothing is sent anywhere. See [[settings]].

### What happens when a countdown runs out?

The run ends by itself. The item on screen counts as not answered: no rating, no change to its schedule. Pausing
stops the clock.

### Why do new items start as "difficult"?

So that StudyFrog is careful with an item until you have reviewed it. Choose another difficulty in the Create dialog
or on the item page. See [[create-dialog]].

## Scheduling and familiarity

### What is the difference between FSRS and SM-2?

FSRS (the default) is a modern method that fits your memory better and lets you choose a target retention. SM-2 is
the classic, simpler method. See [[familiarity|Familiarity and scheduling]].

### Can I switch familiarity off?

Yes: `familiarity_strength = 0` in `studyfrog.toml` for the vault, or `familiarity = false` in a stack's
`_stack.toml`. Settings, **Does familiarity help?** tells you from your own history whether it helps you.

### How accurate is the forecast of reviews ahead?

On simulated histories the weekly totals were off by about 10 %. The Reports screen says whether it was checked on
your own history or a simulated one. It is an expectation, not a promise. See [[reports]].

## Planning

### Will StudyFrog remind me when it is closed?

No. Reminders show only while StudyFrog runs. A reminder missed while it was closed shows once when you open it
again, if its entry has not ended. See [[notifications]].

### Can I move reviews to another day?

Yes. Drag a planned-review chip on the [[calendar]] to another day, or use **Move to another day…**. **Undo last
move** takes it back.

### Why are there two boards?

The **Board** screen is a free-form task board. A stack's **Board** tab sorts the stack's items by status, and moving
an item there changes its status. See [[board]].

## The library

### How do I change many items at once?

Tick them in the [[library]] list, then use the bar: status, priority, difficulty, stack, tags, or delete. **Undo last
change** takes a change back.

### Can I write my own searches?

Yes, in the library's **Query** mode, and save them in the **Filters** menu. See [[query-language]].

### My wikilink does not work after I renamed a note.

Wikilinks are matched by title only. Change the links to the new title. See [[graph]].

### Can a teacher's name contain a comma?

Yes, for example "Prof. Dr. Künkelmann, Harry". If a name cannot be stored, Settings tells you why next to the field
before saving.

## PDFs

### Why can I not search or select the text of my scanned PDF?

Scanned pages are pictures without text. StudyFrog says how many pages are affected. Reading text from pictures (OCR)
is not available yet. See [[pdf]].

### Does StudyFrog change my PDF files?

Never. Highlights and notes go into a separate file next to the PDF; rearranged pages are saved as a new file.

## The assistant

### Does StudyFrog send my notes to an AI company?

Only if you connect a provider, tick **Allow sending text from my vault** for it, and press **Send**. Before that the
screen tells you how much text goes where, and you can read it. A model on your own computer (such as Ollama) keeps
everything local. See [[assistant]].

### The assistant does not work.

You need a connection first: add one on the **Connections** tab. The error message says whether the key was refused,
the model name is wrong or the server cannot be reached. The assistant has been tested against the providers'
documented formats only, not yet against the real services, so a provider may still behave differently.

## Other

### Can I add my own card types or importers?

Extensions are built into the program by developers; there is no plugin loading. When an extension is present, a
stack page shows **Import…** with its importers and card types.

### Does StudyFrog work with a screen reader?

Every control has a name, there are buttons for everything you can drag, and texts such as the countdown warnings are
announced. Real screen readers and input methods for other scripts (IME) have not been tested yet.

### Where are the keyboard shortcuts?

See [[keyboard|Keyboard shortcuts]]. **F1** opens help for the screen you are on. See [[help|Using help]].

### Why is a setting not on the Settings screen?

Some settings are kept in text files you edit yourself, such as `studyfrog.toml` and `_stack.toml`. The Settings
screen shows their values and says which file to edit. A typo in such a file is reported with its line. See
[[settings]].
