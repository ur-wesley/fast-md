use crate::services::fs::read_document_file;
use crate::services::workspace::canonical_workspace_key;
use crate::types::{FileFilterMode, TabItem};
use eyre::{Context, Result, eyre};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, Value, STORED, STRING, TEXT};
use tantivy::{doc, Index, IndexWriter, TantivyDocument};
use walkdir::WalkDir;

const MAX_FILE_BYTES: u64 = 1_048_576;
const WRITER_HEAP_BYTES: usize = 50_000_000;
const DEFAULT_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub snippet: String,
}

struct SessionIndex {
    root: PathBuf,
    #[allow(dead_code)]
    filter: FileFilterMode,
    index: Index,
    path_field: Field,
    body_field: Field,
}

static SESSION_INDEX: OnceLock<Mutex<Option<SessionIndex>>> = OnceLock::new();

fn session_lock() -> &'static Mutex<Option<SessionIndex>> {
    SESSION_INDEX.get_or_init(|| Mutex::new(None))
}

fn lock_session() -> Result<std::sync::MutexGuard<'static, Option<SessionIndex>>> {
    session_lock()
        .lock()
        .map_err(|_| eyre!("fts index lock poisoned"))
}

fn build_schema() -> (Schema, Field, Field) {
    let mut schema_builder = Schema::builder();
    let path_field = schema_builder.add_text_field("path", STRING | STORED);
    let body_field = schema_builder.add_text_field("body", TEXT | STORED);
    (schema_builder.build(), path_field, body_field)
}

fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules" || name == "target"
}

fn collect_indexable_files(root: &Path, filter: FileFilterMode) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                e.file_name()
                    .to_str()
                    .is_none_or(|name| !should_skip_dir(name))
            } else {
                true
            }
        })
        .filter_map(Result::ok)
    {
        let path = entry.into_path();
        if path.is_file() && filter.matches_path(&path) {
            files.push(path);
        }
    }
    files
}

fn read_indexable(path: &Path, filter: FileFilterMode) -> Result<Option<(String, String)>> {
    if !filter.matches_path(path) {
        return Ok(None);
    }
    let meta = std::fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if meta.len() > MAX_FILE_BYTES {
        return Ok(None);
    }
    let body = read_document_file(path)?;
    let path_str = path.to_string_lossy().to_string();
    Ok(Some((path_str, body)))
}

fn add_document(
    writer: &mut IndexWriter,
    path_field: Field,
    body_field: Field,
    path_str: &str,
    body: &str,
) -> Result<()> {
    writer
        .add_document(doc!(path_field => path_str.to_string(), body_field => body.to_string()))
        .context("add fts document")?;
    Ok(())
}

fn with_writer<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut IndexWriter, Field, Field) -> Result<()>,
{
    let mut guard = lock_session()?;
    let session = guard
        .as_mut()
        .ok_or_else(|| eyre!("fts index not initialized"))?;
    let mut writer = session
        .index
        .writer(WRITER_HEAP_BYTES)
        .context("create fts writer")?;
    f(&mut writer, session.path_field, session.body_field)?;
    writer.commit().context("commit fts index")?;
    Ok(())
}

/// Drop the in-memory index (session end / folder change).
pub fn drop_index() {
    if let Ok(mut guard) = lock_session() {
        *guard = None;
    }
}

fn canonical_root(path: &Path) -> PathBuf {
    canonical_workspace_key(path).unwrap_or_else(|| path.to_path_buf())
}

/// True when the RAM index matches `root`.
#[must_use]
pub fn is_index_for(root: &Path) -> bool {
    let target = canonical_root(root);
    lock_session()
        .ok()
        .and_then(|guard| {
            guard
                .as_ref()
                .map(|session| canonical_root(&session.root) == target)
        })
        .unwrap_or(false)
}

/// Rebuild the session index for `root` from disk.
pub fn rebuild_root(root: PathBuf, filter: FileFilterMode) -> Result<()> {
    let (schema, path_field, body_field) = build_schema();
    let index = Index::create_in_ram(schema);
    let mut writer = index
        .writer(WRITER_HEAP_BYTES)
        .context("create fts writer")?;

    for path in collect_indexable_files(&root, filter) {
        if let Some((path_str, body)) = read_indexable(&path, filter)? {
            add_document(&mut writer, path_field, body_field, &path_str, &body)?;
        }
    }
    writer.commit().context("commit fts rebuild")?;

    let mut guard = lock_session()?;
    *guard = Some(SessionIndex {
        root: canonical_root(&root),
        filter,
        index,
        path_field,
        body_field,
    });
    Ok(())
}

/// Upsert one file into the active index.
pub fn upsert_path(path: &Path, content: &str) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    with_writer(|writer, path_field, body_field| {
        writer.delete_term(tantivy::Term::from_field_text(path_field, &path_str));
        add_document(writer, path_field, body_field, &path_str, content)
    })
}

/// Search the active index.
pub fn search(query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let guard = lock_session()?;
    let session = guard
        .as_ref()
        .ok_or_else(|| eyre!("fts index not initialized"))?;

    let reader = session
        .index
        .reader()
        .context("create fts reader")?;
    let searcher = reader.searcher();

    let parser = QueryParser::for_index(&session.index, vec![session.body_field]);
    let parsed = parser
        .parse_query(trimmed)
        .context("parse fts query")?;

    let top_docs = searcher
        .search(&parsed, &TopDocs::with_limit(limit))
        .context("fts search")?;

    let mut hits = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc: TantivyDocument = searcher.doc(doc_address).context("load fts doc")?;
        let path_value = doc
            .get_first(session.path_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("fts doc missing path"))?;
        let body = doc
            .get_first(session.body_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let snippet = make_snippet(body, trimmed);
        hits.push(SearchHit {
            path: PathBuf::from(path_value),
            snippet,
        });
    }
    Ok(hits)
}

fn matches_content(content: &str, query: &str) -> bool {
    let content_lower = content.to_lowercase();
    query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .all(|term| content_lower.contains(&term.to_lowercase()))
}

fn make_snippet(content: &str, query: &str) -> String {
    let content_lower = content.to_lowercase();
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(|term| term.to_lowercase())
        .collect();
    let Some(first_term) = terms.first() else {
        return String::new();
    };
    let Some(pos) = content_lower.find(first_term) else {
        return String::new();
    };
    let start = pos.saturating_sub(40);
    let end = (pos + first_term.len() + 80).min(content.len());
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
    }
    snippet.push_str(&content[start..end]);
    if end < content.len() {
        snippet.push('…');
    }
    snippet
}

/// Search the index and merge open-tab RAM content (dirty / untitled tabs).
pub fn search_all(
    query: &str,
    limit: usize,
    tabs: &[TabItem],
    has_index: bool,
) -> Result<Vec<SearchHit>> {
    let effective_limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
    let mut hits = if has_index {
        search(query, effective_limit.saturating_mul(2))?
    } else {
        Vec::new()
    };

    let indexed_paths: HashSet<PathBuf> = hits.iter().map(|hit| hit.path.clone()).collect();

    for tab in tabs {
        if tab.content.is_empty() || !matches_content(&tab.content, query) {
            continue;
        }

        let path = match &tab.path {
            Some(path) if !tab.is_dirty && indexed_paths.contains(path) => continue,
            Some(path) => path.clone(),
            None => PathBuf::from(&tab.title),
        };

        let snippet = make_snippet(&tab.content, query);
        if let Some(pos) = hits.iter().position(|hit| hit.path == path) {
            hits[pos].snippet = snippet;
        } else {
            hits.push(SearchHit { path, snippet });
        }
    }

    hits.truncate(effective_limit);
    Ok(hits)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fast_md_fts_{name}_{nanos}"));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_rebuild_search_upsert_and_drop() {
        drop_index();
        let dir = temp_dir("basic");
        let alpha = dir.join("alpha.md");
        let beta = dir.join("beta.md");
        fs::write(&alpha, "Hello unique-alpha-token in alpha file.").unwrap();
        fs::write(&beta, "Beta file has other content.").unwrap();

        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        assert!(is_index_for(&dir));

        let hits = search("unique-alpha-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        assert!(hits[0].snippet.contains("unique-alpha-token"));

        upsert_path(&alpha, "Hello updated unique-alpha-token content.").unwrap();
        let hits = search("updated", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);

        fs::remove_file(&beta).unwrap();
        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        let hits = search("Beta", 10).unwrap();
        assert!(hits.is_empty());

        drop_index();
        assert!(!is_index_for(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_all_merges_dirty_tab() {
        drop_index();
        let dir = temp_dir("dirty");
        let doc = dir.join("notes.md");
        fs::write(&doc, "disk-only keyword").unwrap();
        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let tab = TabItem {
            id: 1,
            path: Some(doc.clone()),
            title: "notes.md".to_string(),
            content: "dirty-tab unique-ram-token".to_string(),
            parsed: crate::services::markdown::parse_markdown_document("dirty-tab unique-ram-token"),
            is_dirty: true,
            html_revision: 0,
        };

        let hits = search_all("unique-ram-token", 10, std::slice::from_ref(&tab), true).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, doc);

        drop_index();
        let _ = fs::remove_dir_all(&dir);
    }
}
