//! The standard prompts as MCP prompts (STORY-0119): every prompt of the library
//! (`studyfrog_service::prompts::PROMPTS`) with the optional arguments `file_path` (a vault file
//! attached up to `ATTACH_LIMIT` characters) and `target_stack`, plus `depth` for `explain` and
//! `count` for `exam`.

use std::fs;
use std::sync::OnceLock;

use serde_json::Value;
use studyfrog_agent::mcp_types::{
    McpError, McpPrompt, McpPromptArgument, McpPromptMessage, McpRegistry, register_prompt,
};
use studyfrog_service::prompts::{PROMPTS, PromptSource, clip, prompt_with, run_message};
use studyfrog_service::prompts_meta::params_of;

use crate::mcp_vault::{VaultCtx, arg_str, internal, invalid};
use crate::mcp_vault_read::find_stack;
use crate::mcp_write::stacks;

const FILE: McpPromptArgument = McpPromptArgument {
    name: "file_path",
    description: "A file in the vault (such as notes/cells.md) whose text is attached",
    required: false,
};
const STACK: McpPromptArgument = McpPromptArgument {
    name: "target_stack",
    description: "The stack (folder or title) the produced items go into",
    required: false,
};
const COMMON: &[McpPromptArgument] = &[FILE, STACK];
const EXPLAIN: &[McpPromptArgument] = &[
    FILE,
    STACK,
    McpPromptArgument {
        name: "depth",
        description: "all, intuitive, core or technical",
        required: false,
    },
];
const EXAM: &[McpPromptArgument] = &[
    FILE,
    STACK,
    McpPromptArgument {
        name: "count",
        description: "How many questions: 3 or 5",
        required: false,
    },
];

/// The option of a choice whose name contains `wanted` (ignoring case), as (key, index).
fn pick(id: &str, key: &'static str, wanted: &str) -> Option<(&'static str, usize)> {
    let param = params_of(id).iter().find(|p| p.key == key)?;
    let wanted = wanted.to_lowercase();
    param
        .options
        .iter()
        .position(|(name, text)| name.to_lowercase().contains(&wanted) || *text == wanted)
        .map(|at| (key, at))
}

/// The text of a vault file, if it is inside the vault.
fn attached(ctx: &VaultCtx, rel: &str) -> Result<String, McpError> {
    let root = &ctx.session.root;
    let full = root.join(rel);
    let inside =
        matches!((full.canonicalize(), root.canonicalize()), (Ok(f), Ok(b)) if f.starts_with(&b));
    if !inside || rel.starts_with('/') {
        return Err(invalid(format!("{rel} is not a file in the vault")));
    }
    fs::read_to_string(&full).map_err(internal)
}

fn messages_of(
    ctx: &mut VaultCtx,
    source: &PromptSource,
    args: &Value,
) -> Result<Vec<McpPromptMessage>, McpError> {
    let picks: Vec<(&str, usize)> = [("depth", "depth"), ("count", "count")]
        .into_iter()
        .filter_map(|(key, arg)| arg_str(args, arg).and_then(|w| pick(source.id, key, w)))
        .collect();
    let mut prompt = prompt_with(source, &picks);
    if let Some(wanted) = arg_str(args, "target_stack") {
        let all = stacks(ctx)?;
        let stack =
            find_stack(&all, wanted).ok_or_else(|| invalid(format!("no stack {wanted}")))?;
        prompt.text.push_str(&format!(
            "\n\nPut every item into the stack `{}`: write that folder in the stack column.",
            stack.folder
        ));
    }
    let text = match arg_str(args, "file_path") {
        None => prompt.text,
        Some(rel) => {
            let (body, cut) = clip(&attached(ctx, rel)?);
            let note = if cut {
                "\n\n(The file was cut to its first part.)"
            } else {
                ""
            };
            format!("{}{note}", run_message(&prompt, rel, &body))
        }
    };
    Ok(vec![McpPromptMessage { role: "user", text }])
}

/// The summary line of every prompt, worked out once.
fn summaries() -> &'static [String] {
    static SUMMARIES: OnceLock<Vec<String>> = OnceLock::new();
    SUMMARIES.get_or_init(|| {
        PROMPTS
            .iter()
            .map(|p| studyfrog_service::prompts::prompt_of(p).summary)
            .collect()
    })
}

macro_rules! prompt_fn {
    ($name:ident, $at:expr) => {
        fn $name(ctx: &mut VaultCtx, args: &Value) -> Result<Vec<McpPromptMessage>, McpError> {
            messages_of(ctx, &PROMPTS[$at], args)
        }
    };
}
prompt_fn!(p0, 0);
prompt_fn!(p1, 1);
prompt_fn!(p2, 2);
prompt_fn!(p3, 3);
prompt_fn!(p4, 4);
prompt_fn!(p5, 5);
prompt_fn!(p6, 6);
prompt_fn!(p7, 7);
prompt_fn!(p8, 8);
prompt_fn!(p9, 9);

type PromptFn = fn(&mut VaultCtx, &Value) -> Result<Vec<McpPromptMessage>, McpError>;
const GETS: [PromptFn; 10] = [p0, p1, p2, p3, p4, p5, p6, p7, p8, p9];

/// Register every standard prompt.
pub fn register(mut reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    for (at, source) in PROMPTS.iter().enumerate().take(GETS.len()) {
        let arguments = match source.id {
            "explain" => EXPLAIN,
            "exam" => EXAM,
            _ => COMMON,
        };
        reg = register_prompt(
            reg,
            McpPrompt {
                name: source.id,
                description: summaries()[at].as_str(),
                arguments,
                get: GETS[at],
            },
        );
    }
    reg
}
