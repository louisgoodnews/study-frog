//! Matching an address against a pattern such as `studyfrog://items/{id}`. Pure.

/// The parts of `uri` that match the `{name}` parts of `template`, as (name, value), or `None` when
/// it does not match. A part is not empty and does not contain `/`. A template without parts
/// matches only itself.
pub fn match_template(template: &str, uri: &str) -> Option<Vec<(String, String)>> {
    let mut found = Vec::new();
    let (mut pattern, mut rest) = (template, uri);
    while let Some(open) = pattern.find('{') {
        rest = rest.strip_prefix(&pattern[..open])?;
        let close = pattern[open..].find('}')? + open;
        let name = &pattern[open + 1..close];
        pattern = &pattern[close + 1..];
        let end = match pattern.chars().next() {
            Some(next) => rest.find(next)?,
            None => rest.len(),
        };
        let value = &rest[..end];
        if value.is_empty() || value.contains('/') {
            return None;
        }
        found.push((name.to_owned(), value.to_owned()));
        rest = &rest[end..];
    }
    (rest == pattern).then_some(found)
}

/// True when the template has no `{name}` part.
pub fn is_fixed(template: &str) -> bool {
    !template.contains('{')
}
