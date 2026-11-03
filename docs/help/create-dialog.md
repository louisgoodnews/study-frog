# Creating items

New stacks, notes, cards and questions are made in the Create dialog. Open it with **+ Create** in the top bar,
**New item** on a [[stack-page|stack page]], or **New stack…** in the [[library]].

## The four kinds

- **Stack**: a container, like a deck or a notebook. It is a folder in your vault. Stacks can sit inside stacks.
- **Note**: free Markdown text. Link it to other notes with `[[wikilinks]]`.
- **Card**: a flashcard with a **Front** and a **Back**. You flip it while rehearsing.
- **Question**: a **Prompt** and a **Reference answer**. You answer it by typing, and StudyFrog compares your answer
  with the reference. See [[rehearse]].

## Filling in the dialog

1. **Top left corner**: choose the parent stack and the type (Stack, Note, Card or Question). Every note, card and
   question lives in a stack. If there is no stack yet, create one first. A stack can be at the top level ("None
   (top level)") or inside another.
2. **Title**: a large field at the top. For a card or question it is optional: it is taken from the text if you
   leave it empty.
3. **The Markdown editors**: the body of a note, the front and back of a card, the prompt and reference answer of a
   question, or the description of a stack. They have the same toolbar, **Write**, **Split** and **Preview** as on
   the [[item-page|item page]], and a bar underneath that makes each one taller.
4. **Priority**, **Difficulty** and **Tags**.
5. For a stack: a **Due date (optional)**, picked from a month grid, and reminders under **Remind me:**.
6. Press **Create**. Tick **Create another** first to keep the dialog open for the next item.

## Defaults

- A new card, note or question starts with priority **medium** and difficulty **difficult** unless you choose
  others. Difficult is the highest; it makes StudyFrog careful with a new item until you have reviewed it.
- A stack gets no priority or difficulty of its own.
- An item without its own value takes the value of the stack above it.

## A stack with a due date

A due date puts a deadline on the [[calendar]], with the reminders you chose. Once the date has passed, the stack is
no longer suggested for rehearsal and is left out of the counts on Home. You can change the date later on the
stack page.

## Board cards

Every item made in the dialog also gets a card in the first column of the [[board]], so you can plan work on it.

## Size of the dialog

- **Enlarge the dialog** and **Shrink the dialog** at the top right switch between a normal and a wide dialog.
  Notes open wide.
- Drag the grip at the lower left to make the dialog wider, or focus it and use the Left and Right arrow keys.

## The More menu

- **Import or export…** opens the import and export dialog. See [[import-export]].
- **Add PDF ...** copies a PDF file into the chosen stack. See [[pdf]].

## Other ways to create items

- Import them from a file, an Anki package or an Obsidian vault: [[import-export]].
- Make notes, cards and questions from a PDF: [[pdf]].
- Let the [[assistant]] propose them.
- Write Markdown files into the vault folder yourself, with the same front matter StudyFrog writes (copy an
  existing file to see it). StudyFrog notices new and changed files within a second or two. A file it cannot read
  is listed under **Problems** in the library's **Overview**.

See also [[getting-started|Getting started]] and the [[faq]].
