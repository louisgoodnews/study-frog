//! Front-matter fences and top-level key blocks. Pure text slicing; borrows the input.

/// A file split into its YAML front-matter and Markdown body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split<'a> {
    /// Text between the fences (may be empty).
    pub yaml: &'a str,
    /// Everything after the closing fence line.
    pub body: &'a str,
}

/// A top-level YAML key with its verbatim text (key line plus continuation lines).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block<'a> {
    /// Key name without quotes.
    pub key: &'a str,
    /// Verbatim text, ending with the newline of its last line when present.
    pub text: &'a str,
    /// 1-based line within the YAML region.
    pub line: usize,
}

fn is_fence(line: &str) -> bool {
    line.trim_end_matches(['\n', '\r']).trim_end() == "---"
}

/// Split `---\n<yaml>---\n<body>`. `None` when the file does not start with a fence or the
/// closing fence is missing. A leading BOM, CRLF line endings and a closing fence at end of
/// file without a trailing newline are accepted.
pub fn split_front_matter(text: &str) -> Option<Split<'_>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if !is_fence(first) {
        return None;
    }
    let yaml_start = first.len();
    let mut offset = yaml_start;
    for line in lines {
        if is_fence(line) {
            return Some(Split {
                yaml: &text[yaml_start..offset],
                body: &text[offset + line.len()..],
            });
        }
        offset += line.len();
    }
    None
}

fn key_of(line: &str) -> Option<&str> {
    let first = line.chars().next()?;
    if first.is_whitespace() || matches!(first, '#' | '-' | '?' | '[' | '{') {
        return None;
    }
    let colon = line.find(':')?;
    let after = line[colon + 1..].chars().next();
    let separated = after.is_none_or(|c| c.is_whitespace());
    separated.then(|| line[..colon].trim_matches(['"', '\'']))
}

/// Split the YAML region into top-level key blocks. Comments, blank lines and indented or
/// `- ` lines belong to the preceding key. Leading text before the first key is dropped.
pub fn split_blocks(yaml: &str) -> Vec<Block<'_>> {
    let mut starts: Vec<(usize, usize, &str)> = Vec::new();
    let mut offset = 0;
    for (i, line) in yaml.split_inclusive('\n').enumerate() {
        if let Some(key) = key_of(line) {
            starts.push((offset, i + 1, key));
        }
        offset += line.len();
    }
    let ends = starts
        .iter()
        .skip(1)
        .map(|s| s.0)
        .chain(std::iter::once(yaml.len()));
    starts
        .iter()
        .zip(ends)
        .map(|(&(start, line, key), end)| Block {
            key,
            text: &yaml[start..end],
            line,
        })
        .collect()
}
