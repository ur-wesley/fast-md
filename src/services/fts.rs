use crate::services::fs::read_document_file;
use crate::services::workspace::canonical_workspace_key;
use crate::types::{FileFilterMode, TabItem};
use eyre::{Context, Result, eyre};
use std::cmp::Ordering;
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
const FILENAME_BOOST: f32 = 3.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub snippet: String,
    pub match_start: usize,
    pub match_len: usize,
    pub name_match: bool,
}

struct SessionIndex {
    root: PathBuf,
    #[allow(dead_code)]
    filter: FileFilterMode,
    index: Index,
    path_field: Field,
    filename_field: Field,
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

fn build_schema() -> (Schema, Field, Field, Field) {
    let mut schema_builder = Schema::builder();
    let path_field = schema_builder.add_text_field("path", STRING | STORED);
    let filename_field = schema_builder.add_text_field("filename", TEXT | STORED);
    let body_field = schema_builder.add_text_field("body", TEXT | STORED);
    (
        schema_builder.build(),
        path_field,
        filename_field,
        body_field,
    )
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

fn filename_of(path_str: &str) -> String {
    Path::new(path_str)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn read_indexable(path: &Path, filter: FileFilterMode) -> Result<Option<(String, String, String)>> {
    if !filter.matches_path(path) {
        return Ok(None);
    }
    let meta = std::fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if meta.len() > MAX_FILE_BYTES {
        return Ok(None);
    }
    let body = read_document_file(path)?;
    let path_str = path.to_string_lossy().to_string();
    let filename = filename_of(&path_str);
    Ok(Some((path_str, filename, body)))
}

fn add_document(
    writer: &mut IndexWriter,
    path_field: Field,
    filename_field: Field,
    body_field: Field,
    path_str: &str,
    filename: &str,
    body: &str,
) -> Result<()> {
    writer
        .add_document(doc!(
            path_field => path_str.to_string(),
            filename_field => filename.to_string(),
            body_field => body.to_string()
        ))
        .context("add fts document")?;
    Ok(())
}

fn with_writer<F>(f: F) -> Result<()>
where
    F: FnOnce(&mut IndexWriter, Field, Field, Field) -> Result<()>,
{
    let mut guard = lock_session()?;
    let session = guard
        .as_mut()
        .ok_or_else(|| eyre!("fts index not initialized"))?;
    let mut writer = session
        .index
        .writer(WRITER_HEAP_BYTES)
        .context("create fts writer")?;
    f(
        &mut writer,
        session.path_field,
        session.filename_field,
        session.body_field,
    )?;
    writer.commit().context("commit fts index")?;
    Ok(())
}

/// Drop the in-memory index (session end / folder closed).
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
///
/// The previous index stays queryable until this swap commits.
pub fn rebuild_root(root: PathBuf, filter: FileFilterMode) -> Result<()> {
    let (schema, path_field, filename_field, body_field) = build_schema();
    let index = Index::create_in_ram(schema);
    let mut writer = index
        .writer(WRITER_HEAP_BYTES)
        .context("create fts writer")?;

    for path in collect_indexable_files(&root, filter) {
        if let Some((path_str, filename, body)) = read_indexable(&path, filter)? {
            add_document(
                &mut writer,
                path_field,
                filename_field,
                body_field,
                &path_str,
                &filename,
                &body,
            )?;
        }
    }
    writer.commit().context("commit fts rebuild")?;

    let mut guard = lock_session()?;
    *guard = Some(SessionIndex {
        root: canonical_root(&root),
        filter,
        index,
        path_field,
        filename_field,
        body_field,
    });
    Ok(())
}

/// Upsert one file into the active index.
pub fn upsert_path(path: &Path, content: &str) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    let filename = filename_of(&path_str);
    with_writer(|writer, path_field, filename_field, body_field| {
        writer.delete_term(tantivy::Term::from_field_text(path_field, &path_str));
        add_document(
            writer,
            path_field,
            filename_field,
            body_field,
            &path_str,
            &filename,
            content,
        )
    })
}

fn query_matches_name(filename: &str, query: &str) -> bool {
    let name_lower = filename.to_lowercase();
    query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .any(|term| name_lower.contains(&term.to_lowercase()))
}

fn hit_in_root(path: &Path, root: Option<&Path>) -> bool {
    let Some(root) = root else {
        return true;
    };
    path.starts_with(root) || path.starts_with(canonical_root(root))
}

/// Search the active index.
pub fn search(query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    search_in(query, limit, None)
}

fn search_in(query: &str, limit: usize, root: Option<&Path>) -> Result<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let guard = lock_session()?;
    let session = guard
        .as_ref()
        .ok_or_else(|| eyre!("fts index not initialized"))?;

    let reader = session.index.reader().context("create fts reader")?;
    let searcher = reader.searcher();

    let mut parser = QueryParser::for_index(
        &session.index,
        vec![session.filename_field, session.body_field],
    );
    parser.set_field_boost(session.filename_field, FILENAME_BOOST);
    let parsed = parser.parse_query(trimmed).context("parse fts query")?;

    let top_docs = searcher
        .search(&parsed, &TopDocs::with_limit(limit))
        .context("fts search")?;

    let mut ranked: Vec<(f32, SearchHit)> = Vec::new();
    for (score, doc_address) in top_docs {
        let doc: TantivyDocument = searcher.doc(doc_address).context("load fts doc")?;
        let path_value = doc
            .get_first(session.path_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("fts doc missing path"))?;
        let path = PathBuf::from(path_value);
        if !hit_in_root(&path, root) {
            continue;
        }
        let filename = doc
            .get_first(session.filename_field)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| filename_of(path_value));
        let body = doc
            .get_first(session.body_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let name_match = query_matches_name(&filename, trimmed);
        let (snippet, match_start, match_len) = make_snippet(body, trimmed);
        ranked.push((
            score,
            SearchHit {
                path,
                snippet,
                match_start,
                match_len,
                name_match,
            },
        ));
    }
    ranked.sort_by(|a, b| {
        b.1.name_match
            .cmp(&a.1.name_match)
            .then(b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal))
    });
    Ok(ranked.into_iter().map(|(_, hit)| hit).collect())
}

fn matches_content(content: &str, query: &str) -> bool {
    let content_lower = content.to_lowercase();
    query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .all(|term| content_lower.contains(&term.to_lowercase()))
}

fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    if i > s.len() {
        return s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn make_snippet(content: &str, query: &str) -> (String, usize, usize) {
    let content_lower = content.to_lowercase();
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(|term| term.to_lowercase())
        .collect();
    let Some(first_term) = terms.first() else {
        return (String::new(), 0, 0);
    };
    let Some(pos) = content_lower.find(first_term.as_str()) else {
        return (String::new(), 0, 0);
    };
    let start = floor_char_boundary(content, pos.saturating_sub(40));
    let end = ceil_char_boundary(content, (pos + first_term.len() + 80).min(content.len()));
    let end = end.max(start);
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
    }
    let match_start = snippet.len() + (pos.saturating_sub(start));
    snippet.push_str(&content[start..end]);
    if end < content.len() {
        snippet.push('…');
    }
    let match_len = first_term.len();
    let match_end = match_start.saturating_add(match_len);
    if match_end > snippet.len()
        || !snippet.is_char_boundary(match_start)
        || !snippet.is_char_boundary(match_end.min(snippet.len()))
    {
        return (snippet, 0, 0);
    }
    (snippet, match_start, match_len)
}

fn apply_snippet(hit: &mut SearchHit, content: &str, query: &str) {
    let (snippet, match_start, match_len) = make_snippet(content, query);
    hit.snippet = snippet;
    hit.match_start = match_start;
    hit.match_len = match_len;
}

fn tab_name_match(tab: &TabItem, query: &str) -> bool {
    let name = tab
        .path
        .as_ref()
        .and_then(|path| path.file_name())
        .map_or(tab.title.as_str(), |name| {
            name.to_str().unwrap_or(tab.title.as_str())
        });
    query_matches_name(name, query)
}

/// Search the index and merge dirty / untitled tab RAM content.
pub fn search_all(
    query: &str,
    limit: usize,
    tabs: &[TabItem],
    has_index: bool,
    root: Option<&Path>,
) -> Result<Vec<SearchHit>> {
    let effective_limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
    let mut hits = if has_index {
        search_in(query, effective_limit.saturating_mul(2), root)?
    } else {
        Vec::new()
    };

    for tab in tabs {
        if tab.content.is_empty() || !matches_content(&tab.content, query) {
            continue;
        }

        match &tab.path {
            Some(path) if !tab.is_dirty => {
                if let Some(existing) = hits.iter_mut().find(|hit| &hit.path == path) {
                    apply_snippet(existing, &tab.content, query);
                }
            }
            Some(path) => {
                if !hit_in_root(path, root) {
                    continue;
                }
                if let Some(existing) = hits.iter_mut().find(|hit| &hit.path == path) {
                    apply_snippet(existing, &tab.content, query);
                    existing.name_match = existing.name_match || tab_name_match(tab, query);
                } else {
                    let (snippet, match_start, match_len) = make_snippet(&tab.content, query);
                    hits.push(SearchHit {
                        path: path.clone(),
                        snippet,
                        match_start,
                        match_len,
                        name_match: tab_name_match(tab, query),
                    });
                }
            }
            None => {
                let (snippet, match_start, match_len) = make_snippet(&tab.content, query);
                hits.push(SearchHit {
                    path: PathBuf::from(&tab.title),
                    snippet,
                    match_start,
                    match_len,
                    name_match: tab_name_match(tab, query),
                });
            }
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
    use std::sync::{Mutex, MutexGuard};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn serial() -> MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

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
        let _guard = serial();
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
        assert!(hits[0].match_len > 0);

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
    fn test_search_filename_without_body_match() {
        let _guard = serial();
        drop_index();
        let dir = temp_dir("filename");
        let named = dir.join("specialtoken.md");
        fs::write(&named, "nothing relevant in the body at all.").unwrap();
        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let hits = search("specialtoken", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, named);
        assert!(hits[0].name_match);

        drop_index();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_without_drop_keeps_index() {
        let _guard = serial();
        drop_index();
        let dir = temp_dir("nodrop");
        let alpha = dir.join("alpha.md");
        fs::write(&alpha, "keep-me-token lives here.").unwrap();
        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let dir_clone = dir.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(5));
            rebuild_root(dir_clone, FileFilterMode::MarkdownOnly)
        });
        let hits = search("keep-me-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        handle.join().unwrap().unwrap();
        assert!(is_index_for(&dir));
        let hits = search("keep-me-token", 10).unwrap();
        assert_eq!(hits.len(), 1);

        drop_index();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_all_merges_dirty_tab() {
        let _guard = serial();
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
            parse_gen: 0,
            parse_status: crate::types::ParseStatus::Ready,
        };

        let hits = search_all(
            "unique-ram-token",
            10,
            std::slice::from_ref(&tab),
            true,
            Some(&dir),
        )
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, doc);

        drop_index();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_all_skips_clean_tab_not_in_index() {
        let _guard = serial();
        drop_index();
        let dir = temp_dir("cleantab");
        let indexed = dir.join("in-index.md");
        fs::write(&indexed, "index-token").unwrap();
        rebuild_root(dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let outside = dir.join("not-opened-on-disk.md");
        let tab = TabItem {
            id: 1,
            path: Some(outside.clone()),
            title: "not-opened-on-disk.md".to_string(),
            content: "index-token also in this clean tab".to_string(),
            parsed: crate::services::markdown::parse_markdown_document(
                "index-token also in this clean tab",
            ),
            is_dirty: false,
            html_revision: 0,
            parse_gen: 0,
            parse_status: crate::types::ParseStatus::Ready,
        };

        let hits = search_all("index-token", 10, std::slice::from_ref(&tab), true, Some(&dir))
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, indexed);

        drop_index();
        let _ = fs::remove_dir_all(&dir);
    }
}
