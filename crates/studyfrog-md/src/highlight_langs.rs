//! The languages the highlighter knows, as plain data: what starts a comment or a string and which words are
//! keywords. A new language is one more record (the owner's list ends with "etc.").

/// How a language is tokenized.
#[derive(Debug, Clone, Copy)]
pub struct Lang {
    /// The names a code fence may use, lower case.
    pub names: &'static [&'static str],
    /// Words drawn as keywords (case-sensitive).
    pub keywords: &'static [&'static str],
    /// What starts a comment that runs to the end of the line.
    pub line_comments: &'static [&'static str],
    /// A comment with a start and an end.
    pub block_comment: Option<(&'static str, &'static str)>,
    /// The characters that quote a string.
    pub quotes: &'static [char],
    /// Triple quotes make a string that may span lines (Python).
    pub triple_quotes: bool,
    /// Numbers are drawn as numbers.
    pub numbers: bool,
    /// How the rest of the language is read, when it is not a program.
    pub flavour: Flavour,
}

/// Languages that need more than comments, strings, numbers and words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    /// A program: the generic rules.
    Code,
    /// Tags in angle brackets are keywords.
    Markup,
    /// The first line names the columns.
    Table,
    /// A line in square brackets is a section header.
    Sections,
}

const PLAIN: Lang = Lang {
    names: &[],
    keywords: &[],
    line_comments: &[],
    block_comment: None,
    quotes: &[],
    triple_quotes: false,
    numbers: false,
    flavour: Flavour::Code,
};

const C_LIKE: Lang = Lang {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\''],
    numbers: true,
    ..PLAIN
};

/// Every language.
pub const LANGS: &[Lang] = &[
    Lang {
        names: &["python", "py"],
        keywords: crate::highlight_words::PYTHON,
        line_comments: &["#"],
        quotes: &['"', '\''],
        triple_quotes: true,
        numbers: true,
        ..PLAIN
    },
    Lang {
        names: &["r"],
        keywords: crate::highlight_words::R,
        line_comments: &["#"],
        quotes: &['"', '\''],
        numbers: true,
        ..PLAIN
    },
    Lang {
        names: &["rust", "rs"],
        keywords: crate::highlight_words::RUST,
        quotes: &['"'],
        ..C_LIKE
    },
    Lang {
        names: &["php"],
        keywords: crate::highlight_words::PHP,
        line_comments: &["//", "#"],
        ..C_LIKE
    },
    Lang {
        names: &["java"],
        keywords: crate::highlight_words::JAVA,
        ..C_LIKE
    },
    Lang {
        names: &["cpp", "c++", "cc", "cxx", "c", "h", "hpp"],
        keywords: crate::highlight_words::CPP,
        ..C_LIKE
    },
    Lang {
        names: &["toml"],
        keywords: &["true", "false"],
        line_comments: &["#"],
        quotes: &['"', '\''],
        numbers: true,
        flavour: Flavour::Sections,
        ..PLAIN
    },
    Lang {
        names: &["json", "jsonc"],
        keywords: &["true", "false", "null"],
        quotes: &['"'],
        numbers: true,
        ..PLAIN
    },
    Lang {
        names: &["html", "xml", "htm"],
        block_comment: Some(("<!--", "-->")),
        quotes: &['"', '\''],
        flavour: Flavour::Markup,
        ..PLAIN
    },
    Lang {
        names: &["csv", "tsv"],
        quotes: &['"'],
        numbers: true,
        flavour: Flavour::Table,
        ..PLAIN
    },
];

/// The language a code fence names (its first word, in any letter case).
pub fn lang_of(info: &str) -> Option<&'static Lang> {
    let word = info.split_whitespace().next()?.to_ascii_lowercase();
    LANGS.iter().find(|l| l.names.contains(&word.as_str()))
}
