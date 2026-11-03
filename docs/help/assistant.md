# Assistant

The assistant connects StudyFrog to an AI model. It can read stacks you choose and propose new notes, cards and
questions or edits, which you accept one by one. It can also examine you. The assistant is optional: StudyFrog works
fully without it. Open **Assistant** in the sidebar.

The screen has four tabs: **Connections**, **Work the vault**, **Exam me** and **Prompts**. The Prompts tab holds
ready-made prompts for any AI chat, to copy or save: see [[prompts]].

## Connections

A connection is a model the assistant talks to.

- **A model on this computer**, such as Ollama and similar tools. Nothing leaves your computer.
- **A model at a provider**: OpenAI-compatible services or Anthropic.

To add one, fill in **Connection name**, the kind of service, the **Address**, the **Model** name and, for a
provider, the **API key** (not needed for a local model), then press **Add connection**. **Use** chooses the
connection to work with; **Remove** deletes it.

A connection to a provider is refused until you tick **Allow sending text from my vault** for it.

API keys are kept in a private file in your config folder, never in the vault. They are not encrypted. Errors say
what went wrong (a refused key, a wrong model name, a server that cannot be reached) and never show your key.

## Work the vault

1. Tick the **Stacks the assistant may read**.
2. Write **What should the assistant do?**, for example "Make ten cards about the irregular verbs".
3. StudyFrog says how many items and characters would be sent and where (at most 30 000 characters, 3 000 per
   item). Press **Show the text that will be sent** to read it first.
4. Press **Send**. A model on this computer is asked straight away; a provider only after you press Send.
5. Wait for the **Proposal**. It lists each change: a new note, card or question in a stack, or a change to an
   existing item. Changes that break the rules (unknown stacks or items, too long, too many) are listed as rejected.
6. Tick **Accept** on the changes you want and press **Apply accepted changes**. **Discard proposal** throws it
   all away.

Nothing is written to your vault until you apply. A change that fails does not stop the others. Items made by the
assistant do not get a board card.

From the [[pdf|PDF viewer]] you can **Ask the assistant about these pages**: the chosen pages (at most 20) are added
to what the assistant reads, and the cards and questions it proposes keep a link to their page. **Stop reading it**
removes the PDF again.

## Exam me

1. Choose stacks and the **Number of questions**, then press **Start exam**.
2. The **Examiner** asks one question at a time from your cards and questions (not notes). Type **Your answer** and
   press **Send answer**. The model grades each answer.
3. At the end you see the **Result**. Press **Record as reviews** to save the verdicts as reviews: correct counts as
   Good, partly right as Hard, not right as Again. This updates your schedule. Nothing is recorded without that
   click.

## Good to know

- The assistant only proposes. You decide what is applied.
- A model can be wrong. Check what it proposes and how it grades.
- Every call is logged on your computer with its time, connection, address and status, never with your text or
  keys. See [[privacy]].
- The assistant can also propose help pages of your own; see [[help|Using help]].

## Related pages

[[privacy]], [[settings]], [[faq]].
