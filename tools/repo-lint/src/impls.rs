//! P1 check: no inherent `impl` blocks, including ones generated inside macros.
//!
//! Trait impls are allowed. A reviewed exception needs `// allow-impl: <reason>` on the line
//! directly above the block.

use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::Visit;
use syn::{ItemImpl, Macro};

/// One violation: 1-based line and a short description.
pub struct Finding {
    pub line: usize,
    pub message: &'static str,
}

struct Collector<'a> {
    lines: Vec<&'a str>,
    found: Vec<Finding>,
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        record_impl(self, node);
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        record_macro(self, node);
        syn::visit::visit_macro(self, node);
    }
}

/// Check one Rust source file. Unparseable files yield a single finding.
pub fn check_file(path: &Path) -> Vec<Finding> {
    let Ok(source) = fs::read_to_string(path) else {
        return vec![finding(1, "cannot read file")];
    };
    match syn::parse_file(&source) {
        Ok(ast) => collect(&source, &ast),
        Err(_) => vec![finding(1, "cannot parse file")],
    }
}

fn collect(source: &str, ast: &syn::File) -> Vec<Finding> {
    let mut collector = Collector {
        lines: source.lines().collect(),
        found: Vec::new(),
    };
    collector.visit_file(ast);
    collector.found
}

fn record_impl(c: &mut Collector, node: &ItemImpl) {
    let line = node.impl_token.span.start().line;
    if node.trait_.is_none() && !is_allowed(&c.lines, line) {
        c.found.push(finding(
            line,
            "inherent impl block (P1: use free functions)",
        ));
    }
}

fn record_macro(c: &mut Collector, node: &Macro) {
    let line = node
        .path
        .segments
        .first()
        .map_or(1, |s| s.ident.span().start().line);
    if macro_has_inherent_impl(node.tokens.clone()) && !is_allowed(&c.lines, line) {
        c.found.push(finding(
            line,
            "macro body contains an inherent impl block (P1)",
        ));
    }
}

fn finding(line: usize, message: &'static str) -> Finding {
    Finding { line, message }
}

/// True when the line above (1-based `line`) carries the exemption marker.
fn is_allowed(lines: &[&str], line: usize) -> bool {
    line.checked_sub(2)
        .and_then(|i| lines.get(i))
        .is_some_and(|l| l.contains("allow-impl:"))
}

/// Heuristic: `impl` token without a following `for` at the same token level.
fn macro_has_inherent_impl(tokens: TokenStream) -> bool {
    let flat: Vec<TokenTree> = tokens.into_iter().collect();
    let nested = flat
        .iter()
        .any(|t| matches!(t, TokenTree::Group(g) if macro_has_inherent_impl(g.stream())));
    nested || (has_ident(&flat, "impl") && !has_ident(&flat, "for"))
}

fn has_ident(tokens: &[TokenTree], name: &str) -> bool {
    tokens
        .iter()
        .any(|t| matches!(t, TokenTree::Ident(i) if i == name))
}
