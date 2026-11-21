//! The language part of the FrogScript reference: syntax, operators and control flow. Checked against
//! the code by `tests/wiki.rs`, which also reads the keywords from `token.rs`.

use crate::wiki::{Category, Topic, topic_entry as t};

const S: Category = Category::Syntax;
const O: Category = Category::Operators;
const C: Category = Category::ControlFlow;

/// The syntax, operator and control-flow entries.
pub const LANGUAGE: &[Topic] = &[
    t!(
        "from",
        S,
        "from stack \"<name>\" | from vault",
        "Collection",
        "Sets the source of the study items: one stack, or the whole vault.",
        "cards = from stack \"French\""
    ),
    t!(
        "stack",
        S,
        "stack \"<name>\"",
        "Collection",
        "Names one stack as the source, inside a pipeline such as from stack.",
        "from stack \"French\" | limit 10"
    ),
    t!(
        "type",
        S,
        "type \"<kind>\"",
        "Collection",
        "Keeps the items of one kind (note, card or question), as the type function does in a condition.",
        "from vault | type \"card\""
    ),
    t!(
        "where",
        S,
        "where <condition>",
        "Collection",
        "Keeps the items for which the condition is true.",
        "from vault | where familiarity < 0.7"
    ),
    t!(
        "set",
        S,
        "set <property> <value>",
        "Collection",
        "Changes a property of the items (priority, difficulty, status).",
        "from stack \"French\" | set priority \"high\""
    ),
    t!(
        "tag",
        S,
        "tag \"<name>\"",
        "Collection",
        "Adds a tag to the items.",
        "from stack \"French\" | tag \"exam\""
    ),
    t!(
        "untag",
        S,
        "untag \"<name>\"",
        "Collection",
        "Removes a tag from the items.",
        "from vault | untag \"old\""
    ),
    t!(
        "sort",
        S,
        "sort <property> [asc|desc]",
        "Collection",
        "Orders the items by a property, such as due or familiarity.",
        "from vault | sort due asc"
    ),
    t!(
        "limit",
        S,
        "limit <n>",
        "Collection",
        "Keeps the first n items.",
        "from vault | limit 20"
    ),
    t!(
        "select",
        S,
        "select <field>, <field>",
        "Table",
        "Shows only the named fields of each item.",
        "from vault | select title, due"
    ),
    t!(
        "export",
        S,
        "export note \"<path>\"",
        "Nothing",
        "Writes the items to a summary note or a CSV file.",
        "from vault | export note \"exports/week.md\""
    ),
    t!(
        "true",
        S,
        "true",
        "bool",
        "The boolean true, the result of a condition that holds.",
        "set flag true"
    ),
    t!(
        "false",
        S,
        "false",
        "bool",
        "The boolean false, the result of a condition that does not hold.",
        "set flag false"
    ),
    t!(
        "null",
        S,
        "null",
        "Nothing",
        "The empty value, for a field with nothing in it. Written null or None.",
        "x = null"
    ),
    t!(
        "literals",
        S,
        "\"text\" | 3 | 0.5 | f\"Hello {name}\"",
        "text or number",
        "Text in double quotes; whole and decimal numbers; f-strings put the value of a name in braces.",
        "log_info(f\"{len(cards)} cards\")"
    ),
    t!(
        "==",
        O,
        "a == b",
        "bool",
        "True when the two values are equal.",
        "if priority == \"high\":"
    ),
    t!(
        "!=",
        O,
        "a != b",
        "bool",
        "True when the two values differ.",
        "if status != \"done\":"
    ),
    t!(
        "<",
        O,
        "a < b",
        "bool",
        "True when a is less than b. The same for <=, > and >=.",
        "where familiarity < 0.5"
    ),
    t!(
        "arithmetic",
        O,
        "a + b | a - b | a * b | a / b | a % b",
        "number",
        "Add, subtract, multiply, divide and take the remainder. A unary minus negates.",
        "total = 3 * 4 + 1"
    ),
    t!(
        "and",
        O,
        "a and b",
        "bool",
        "True when both sides are true.",
        "where type \"card\" and tag \"exam\""
    ),
    t!(
        "or",
        O,
        "a or b",
        "bool",
        "True when either side is true.",
        "where priority \"high\" or familiarity < 0.3"
    ),
    t!(
        "not",
        O,
        "not a",
        "bool",
        "True when the value is false.",
        "where not tag \"done\""
    ),
    t!(
        "if",
        C,
        "if <condition>:",
        "Nothing",
        "Runs the indented block when the condition holds. Followed by elif and else.",
        "if familiarity < 0.5:\n    priority = \"high\""
    ),
    t!(
        "elif",
        C,
        "elif <condition>:",
        "Nothing",
        "Another condition, checked when the if above was false.",
        "elif familiarity < 0.8:\n    priority = \"medium\""
    ),
    t!(
        "else",
        C,
        "else:",
        "Nothing",
        "The block that runs when no condition above held.",
        "else:\n    priority = \"low\""
    ),
    t!(
        "for",
        C,
        "for <name> in <collection>:",
        "Nothing",
        "Runs the indented block once for each item of the collection.",
        "for stack in vault.stacks:\n    log_info(stack.name)"
    ),
    t!(
        "in",
        C,
        "for <name> in <collection>",
        "Nothing",
        "Used in a for loop to name the collection to walk through.",
        "for item in items:"
    ),
    t!(
        "def",
        C,
        "def <name>(<parameters>):",
        "Nothing",
        "Declares a function. A parameter may have a default value.",
        "def prioritize(min=0.6):\n    items | where familiarity < min"
    ),
    t!(
        "return",
        C,
        "return <value>",
        "any",
        "Ends the function and gives the value back.",
        "def double(x):\n    return x * 2"
    ),
];
