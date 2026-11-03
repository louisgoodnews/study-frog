# Privacy and your data

StudyFrog keeps your data on your computer. There is no account, no cloud service and no tracking. This page says
exactly what is stored where, and when anything can leave your computer.

## What is stored, and where

- **Your content** (stacks, notes, cards, questions, goals) is plain Markdown in your vault folder. You can open,
  edit, copy or delete these files with any editor.
- **Pictures** you add are copied into the vault's `assets/` folder.
- **PDF highlights and notes** are in a plain text file next to each PDF, named like `name.pdf.notes.toml`.
- **Settings of the vault** are in `studyfrog.toml` (vault root) and `_stack.toml` (next to a stack). Teachers
  and subjects are in `subjects.toml`, saved filters in `filters.toml`, your own achievements in
  `achievements.toml`. All are plain text in the vault.
- **Your history** (reviews, runs, how long you looked at an item, the board, the calendar, notifications) is in a
  database in the vault's hidden `.studyfrog` folder. The index of your files is there too; it can always be
  rebuilt from the files.
- **Your personal settings** (the vault you opened last, reading speed, the last Settings tab) are in `app.toml` in
  your config folder (usually `~/.config/studyfrog/`). Each [[glossary|profile]] has its own.
- **API keys** for the assistant are in a separate file in your config folder that only you can read. They are
  never written into the vault. They are not encrypted.

## What leaves your computer

Nothing, by default. StudyFrog works fully offline.

The only exception is the [[assistant]], and only when you set it up:

- A model that runs **on this computer** (for example Ollama) gets the text you choose to send. It stays on your
  computer.
- A model at a **provider** (OpenAI-compatible or Anthropic) is refused until you tick **Allow sending text from my
  vault** for that connection.
- Before anything is sent to a provider, the Assistant screen tells you how many items and characters will be sent,
  and where. You can read the text first. Nothing is sent until you press **Send**.
- Calls are logged on your computer with time, connection, address and status. The log never holds your text or
  your keys.

Links in PDFs to web or mail addresses open your browser only after you press **Open**. See [[pdf]].

## View tracking

The item page records how long you look at an item (active time only; a minute without input stops the clock).
StudyFrog uses this for [[familiarity]]. Only the time and the item are stored, on this computer. You can switch it
off in [[settings|Settings]], **Study and scheduling** tab: untick **Record how long I look at items**.

## Backups

A backup is one `.tar` file with your notes, pictures, PDFs, review history, board, calendar and, if you want, your
settings and API keys. Make one in Settings, **Data** tab, **Back up now…**.

- API keys are only included if you tick **Include API keys**. They are not encrypted inside the backup, so keep
  such a backup safe.
- A restore goes into a new or empty folder. It never overwrites anything.

## Exports

An export writes a part of your library to a file you choose (JSON, CSV or Markdown). The **Advanced** mode also
writes your review history. See [[import-export]].

## Deleting your data

Everything is files. To remove a vault, delete its folder. To remove a profile, use the profile menu; this removes
the profile's settings and keys, not its vault folder.

See also the [[faq]].
