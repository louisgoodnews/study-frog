//! The wikilink graph, read from the index.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};
use crate::items::load_indexed_items;

/// A node of the link graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    /// Item identity.
    pub uid: Uid,
    /// Title.
    pub title: String,
    /// Note, card or question.
    pub kind: studyfrog_core::plan::ItemKind,
}

/// The wikilink graph: items as nodes, one undirected edge per linked pair.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkGraph {
    /// Items in path order.
    pub nodes: Vec<GraphNode>,
    /// Edges as node indices, each pair once (smaller index first), no self-links.
    pub edges: Vec<(u32, u32)>,
}

/// Build the link graph from the index. A link resolves to the items whose title equals its target
/// (ignoring case); links that resolve to nothing are left out.
pub fn load_link_graph(conn: &Connection) -> Result<LinkGraph, StoreError> {
    let items = load_indexed_items(conn)?;
    let mut nodes = Vec::with_capacity(items.len());
    let mut index = std::collections::HashMap::with_capacity(items.len());
    let titles = item_titles(conn)?;
    for it in &items {
        index.insert(it.uid, nodes.len() as u32);
        nodes.push(GraphNode {
            uid: it.uid,
            title: titles.get(&it.uid).cloned().unwrap_or_default(),
            kind: it.kind,
        });
    }
    let sql = "SELECT l.src_uid, i.uid FROM link l JOIN item i ON i.title = l.target COLLATE NOCASE
               WHERE i.kind != 'stack' AND i.uid != l.src_uid";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sql_err)?;
    let mut edges = std::collections::BTreeSet::new();
    for row in rows {
        let (a, b) = row.map_err(sql_err)?;
        let (a, b) = (parse_ulid(&a), parse_ulid(&b));
        if let (Some(a), Some(b)) = (a.and_then(|u| index.get(&u)), b.and_then(|u| index.get(&u))) {
            edges.insert((*a.min(b), *a.max(b)));
        }
    }
    Ok(LinkGraph {
        nodes,
        edges: edges.into_iter().collect(),
    })
}

fn item_titles(conn: &Connection) -> Result<std::collections::HashMap<Uid, String>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid, title FROM item WHERE kind != 'stack'")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sql_err)?;
    let mut map = std::collections::HashMap::new();
    for row in rows {
        let (id, title) = row.map_err(sql_err)?;
        if let Some(uid) = parse_ulid(&id) {
            map.insert(uid, title);
        }
    }
    Ok(map)
}
