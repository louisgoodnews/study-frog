//! The FrogScript cookbook (STORY-0172): ready-to-run recipes for common study tasks. A recipe marked
//! read-only only reads the vault and can be run straight from the reference; the others change items
//! and are only inserted into the editor, to be read and run on purpose. Checked by `tests/cookbook.rs`.

/// One recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recipe {
    /// Its title.
    pub name: &'static str,
    /// What it does, in one sentence.
    pub summary: &'static str,
    /// Whether it only reads the vault.
    pub read_only: bool,
    /// The script.
    pub code: &'static str,
}

/// Every recipe, read-only ones first.
pub const RECIPES: &[Recipe] = &[
    Recipe {
        name: "Overdue review analyzer",
        summary: "Counts the items that are past their due date and lists the first ten.",
        read_only: true,
        code: "# Items whose due date has passed.\noverdue = from vault | where due < today\nlog_info(f\"{len(overdue)} items are overdue\")\noverdue | sort due asc | limit 10\n",
    },
    Recipe {
        name: "Due today",
        summary: "Lists what is due today.",
        read_only: true,
        code: "# Items due today.\ndue_today = from vault | where due == today\nlog_info(f\"{len(due_today)} items are due today\")\n",
    },
    Recipe {
        name: "Weekly progress reporter",
        summary: "Reports how many items are complete and how many are still being learned.",
        read_only: true,
        code: "# A progress line for the week.\ndone = from vault | where status \"complete\"\nlearning = from vault | where status \"learning\"\nlog_info(f\"Complete: {len(done)}, learning: {len(learning)}\")\n",
    },
    Recipe {
        name: "Stack summary",
        summary: "Writes one line per stack with the number of items in it.",
        read_only: true,
        code: "# One line per stack.\nfor stack in vault.stacks:\n    n = len(from stack stack.name)\n    log_info(f\"{stack.name}: {n} items\")\n",
    },
    Recipe {
        name: "Exams in the next two weeks",
        summary: "Counts the exams planned for the next 14 days.",
        read_only: true,
        code: "# Exams planned in the calendar for the next two weeks.\nexams = calendar.upcoming_exams(14)\nlog_info(f\"{len(exams)} exams in the next 14 days\")\n",
    },
    Recipe {
        name: "Tag census",
        summary: "Writes every tag in use, one per line.",
        read_only: true,
        code: "# Every tag in the vault.\nfor t in vault.tags:\n    log_info(t)\n",
    },
    Recipe {
        name: "Backup reminder",
        summary: "Writes a reminder to back up before a large change. The language cannot start a backup itself.",
        read_only: true,
        code: "# FrogScript cannot start a backup; this reminds you to make one.\nlog_warn(\"Back up the vault (Settings, Backup) before a large change.\")\n",
    },
    Recipe {
        name: "Bulk card tagger",
        summary: "Tags every card of a stack with the same tag.",
        read_only: false,
        code: "# Tag every card of the French stack. Change the stack and the tag as you need.\nfrom stack \"French\" | where type \"card\" | tag \"reviewed\"\n",
    },
    Recipe {
        name: "Vocabulary cleaner",
        summary: "Removes the stale tag from the easy cards, so they come back in the normal queue.",
        read_only: false,
        code: "# Easy cards no longer need the stale tag.\nfrom vault | where type \"card\" | where difficulty \"easy\" | untag \"stale\"\n",
    },
    Recipe {
        name: "Priority reset",
        summary: "Sets well-known high-familiarity items back to medium priority.",
        read_only: false,
        code: "# Items you know well do not need the high priority any more.\nfrom vault | where priority \"high\" | where familiarity > 0.9 | set priority \"medium\"\n",
    },
];

/// The recipe with this name, if there is one.
pub fn recipe(name: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.name == name)
}
