use crate::services::settings::get_settings_dir;
use crate::services::workspace::canonical_workspace_key;
use crate::types::{FileFilterMode, TabItem};
use eyre::{Context, Result, eyre};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, Value, STORED, STRING,
};
use tantivy::tokenizer::{LowerCaser, RegexTokenizer, TextAnalyzer};
use tantivy::{doc, Index, IndexWriter, TantivyDocument, Term};
use walkdir::WalkDir;

const MAX_FILE_BYTES: u64 = 1_048_576;
const WRITER_HEAP_BYTES: usize = 15_000_000;
const DEFAULT_LIMIT: usize = 50;
const INDICES_DIR_NAME: &str = "indices";
const META_FILE_NAME: &str = "meta.json";
const TANTIVY_DIR_NAME: &str = "tantivy";
const IDENT_TOKENIZER: &str = "ident";
const TOKENIZER_VERSION: &str = "ident-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub snippet: String,
    pub match_start: usize,
    pub match_len: usize,
    pub name_match: bool,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMeta {
    root: PathBuf,
    filter: String,
    #[serde(default)]
    tokenizer: String,
}

struct SessionIndex {
    root: PathBuf,
    filter: FileFilterMode,
    index: Index,
    path_field: Field,
    filename_field: Field,
    body_field: Field,
}

static SESSION_INDEX: OnceLock<Mutex<Option<SessionIndex>>> = OnceLock::new();
static SEARCH_EPOCH: AtomicU64 = AtomicU64::new(0);
static INDICES_ROOT_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
static REBUILD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn session_lock() -> &'static Mutex<Option<SessionIndex>> {
    SESSION_INDEX.get_or_init(|| Mutex::new(None))
}

fn lock_session() -> Result<std::sync::MutexGuard<'static, Option<SessionIndex>>> {
    session_lock()
        .lock()
        .map_err(|_| eyre!("fts index lock poisoned"))
}

fn lock_rebuild() -> Result<std::sync::MutexGuard<'static, ()>> {
    REBUILD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| eyre!("fts rebuild lock poisoned"))
}

fn override_lock() -> &'static Mutex<Option<PathBuf>> {
    INDICES_ROOT_OVERRIDE.get_or_init(|| Mutex::new(None))
}

pub fn set_indices_root_for_tests(path: Option<PathBuf>) {
    if let Ok(mut guard) = override_lock().lock() {
        *guard = path;
    }
}

fn indices_parent() -> PathBuf {
    override_lock()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(|| get_settings_dir().join(INDICES_DIR_NAME))
}

fn fnv1a_hex(s: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in s.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[must_use]
pub fn workspace_index_id(root: &Path) -> String {
    fnv1a_hex(&canonical_root(root).to_string_lossy())
}

fn index_dir_for(root: &Path) -> PathBuf {
    indices_parent().join(workspace_index_id(root))
}

fn tantivy_dir(dir: &Path) -> PathBuf {
    dir.join(TANTIVY_DIR_NAME)
}

pub fn bump_epoch() -> u64 {
    SEARCH_EPOCH.fetch_add(1, Ordering::Relaxed) + 1
}

#[must_use]
pub fn current_epoch() -> u64 {
    SEARCH_EPOCH.load(Ordering::Relaxed)
}

fn epoch_stale(epoch: u64) -> bool {
    current_epoch() != epoch
}

fn ident_text_options() -> TextOptions {
    TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(IDENT_TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored()
}

fn build_schema() -> (Schema, Field, Field, Field) {
    let mut schema_builder = Schema::builder();
    let ident_text = ident_text_options();
    let path_field = schema_builder.add_text_field("path", STRING | STORED);
    let filename_field = schema_builder.add_text_field("filename", ident_text.clone());
    let body_field = schema_builder.add_text_field("body", ident_text);
    (
        schema_builder.build(),
        path_field,
        filename_field,
        body_field,
    )
}

fn register_ident_tokenizer(index: &Index) -> Result<()> {
    let analyzer = TextAnalyzer::builder(
        RegexTokenizer::new(r"[A-Za-z0-9_]+").map_err(|err| eyre!("ident tokenizer: {err}"))?,
    )
    .filter(LowerCaser)
    .build();
    index.tokenizers().register(IDENT_TOKENIZER, analyzer);
    Ok(())
}

fn ident_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            buf.push(c.to_ascii_lowercase());
        } else if !buf.is_empty() {
            out.push(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

fn ident_token_ranges(s: &str) -> Vec<(usize, usize, String)> {
    let orig: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut buf = String::new();
    for (i, c) in orig.iter().enumerate() {
        if c.is_ascii_alphanumeric() || *c == '_' {
            if buf.is_empty() {
                start = i;
            }
            buf.push(c.to_ascii_lowercase());
        } else if !buf.is_empty() {
            out.push((start, buf.len(), std::mem::take(&mut buf)));
        }
    }
    if !buf.is_empty() {
        out.push((start, buf.len(), buf));
    }
    out
}

fn collapse_marked_spans(orig: &[char], marked: &[bool]) -> Vec<(bool, String)> {
    let mut spans = Vec::new();
    let mut current = false;
    let mut buf = String::new();
    for (i, ch) in orig.iter().enumerate() {
        let hit = marked.get(i).copied().unwrap_or(false);
        if !buf.is_empty() && hit != current {
            spans.push((current, std::mem::take(&mut buf)));
        }
        current = hit;
        buf.push(*ch);
    }
    if !buf.is_empty() {
        spans.push((current, buf));
    }
    if spans.is_empty() {
        spans.push((false, String::new()));
    }
    spans
}

#[must_use]
pub fn token_spans(text: &str, query: &str) -> Vec<(bool, String)> {
    let want: HashSet<String> = ident_tokens(query).into_iter().collect();
    if want.is_empty() {
        return vec![(false, text.to_string())];
    }
    let orig: Vec<char> = text.chars().collect();
    let mut marked = vec![false; orig.len()];
    for (start, len, token) in ident_token_ranges(text) {
        if want.contains(&token) {
            for flag in marked.iter_mut().skip(start).take(len) {
                *flag = true;
            }
        }
    }
    collapse_marked_spans(&orig, &marked)
}

fn fields_of(index: &Index) -> Result<(Field, Field, Field)> {
    let schema = index.schema();
    let path_field = schema
        .get_field("path")
        .map_err(|_| eyre!("fts schema missing path"))?;
    let filename_field = schema
        .get_field("filename")
        .map_err(|_| eyre!("fts schema missing filename"))?;
    let body_field = schema
        .get_field("body")
        .map_err(|_| eyre!("fts schema missing body"))?;
    Ok((path_field, filename_field, body_field))
}

fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules" || name == "target"
}

#[must_use]
pub fn path_is_skipped(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(should_skip_dir)
    })
}

#[must_use]
pub fn path_is_indexable(path: &Path, root: Option<&Path>) -> bool {
    if !path.is_file() || path_is_skipped(path) {
        return false;
    }
    if let Some(root) = root {
        if !hit_in_root(path, Some(root)) {
            return false;
        }
    }
    true
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
    if !filter.matches_path(path) || path_is_skipped(path) {
        return Ok(None);
    }
    let Ok(meta) = std::fs::metadata(path) else {
        return Ok(None);
    };
    let path_str = path.to_string_lossy().to_string();
    let filename = filename_of(&path_str);
    if meta.len() > MAX_FILE_BYTES {
        return Ok(Some((path_str, filename, String::new())));
    }
    let body = std::fs::read_to_string(path).unwrap_or_default();
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

fn with_writer(path: &Path, f: impl FnOnce(&mut IndexWriter, Field, Field, Field) -> Result<()>) -> Result<()> {
    let mut guard = lock_session()?;
    let session = guard
        .as_mut()
        .ok_or_else(|| eyre!("fts index not initialized"))?;
    if !hit_in_root(path, Some(&session.root)) {
        return Ok(());
    }
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

pub fn drop_index() {
    if let Ok(mut guard) = lock_session() {
        *guard = None;
    }
}

fn canonical_root(path: &Path) -> PathBuf {
    canonical_workspace_key(path).unwrap_or_else(|| path.to_path_buf())
}

fn write_meta(dir: &Path, root: &Path, filter: FileFilterMode) -> Result<()> {
    let meta = IndexMeta {
        root: root.to_path_buf(),
        filter: filter.as_str().to_string(),
        tokenizer: TOKENIZER_VERSION.to_string(),
    };
    let json = serde_json::to_string(&meta).context("serialize fts meta")?;
    std::fs::write(dir.join(META_FILE_NAME), json).context("write fts meta")?;
    Ok(())
}

fn read_meta(dir: &Path) -> Result<IndexMeta> {
    let raw = std::fs::read_to_string(dir.join(META_FILE_NAME)).context("read fts meta")?;
    serde_json::from_str(&raw).context("parse fts meta")
}

fn try_open_existing(dir: &Path, root: &Path, filter: FileFilterMode) -> Result<Index> {
    let meta = read_meta(dir)?;
    if canonical_root(&meta.root) != *root {
        return Err(eyre!("fts meta root mismatch"));
    }
    if meta.filter != filter.as_str() {
        return Err(eyre!("fts meta filter mismatch"));
    }
    if meta.tokenizer != TOKENIZER_VERSION {
        return Err(eyre!("fts tokenizer mismatch"));
    }
    let index = Index::open_in_dir(tantivy_dir(dir)).context("open fts index")?;
    register_ident_tokenizer(&index)?;
    Ok(index)
}

fn attach_session(root: PathBuf, filter: FileFilterMode, index: Index) -> Result<()> {
    register_ident_tokenizer(&index)?;
    let (path_field, filename_field, body_field) = fields_of(&index)?;
    let mut guard = lock_session()?;
    *guard = Some(SessionIndex {
        root,
        filter,
        index,
        path_field,
        filename_field,
        body_field,
    });
    Ok(())
}

fn build_fresh_index(dir: &Path, root: &Path, filter: FileFilterMode) -> Result<Index> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).context("clear fts index dir")?;
    }
    let tdir = tantivy_dir(dir);
    std::fs::create_dir_all(&tdir).context("create fts index dir")?;
    let (schema, path_field, filename_field, body_field) = build_schema();
    let index = Index::create_in_dir(&tdir, schema).context("create fts index")?;
    register_ident_tokenizer(&index)?;
    let mut writer = index
        .writer(WRITER_HEAP_BYTES)
        .context("create fts writer")?;
    for path in collect_indexable_files(root, filter) {
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
    write_meta(dir, root, filter)?;
    Ok(index)
}

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

#[must_use]
pub fn is_full_index_for(root: &Path) -> bool {
    let target = canonical_root(root);
    lock_session()
        .ok()
        .and_then(|guard| {
            guard.as_ref().map(|session| {
                canonical_root(&session.root) == target
                    && session.filter == FileFilterMode::AllFiles
            })
        })
        .unwrap_or(false)
}

pub fn rebuild_root(root: PathBuf, filter: FileFilterMode) -> Result<()> {
    let _rebuild = lock_rebuild()?;
    let canon = canonical_root(&root);
    let same_session = is_index_for(&canon);
    let dir = index_dir_for(&canon);
    if let Ok(index) = try_open_existing(&dir, &canon, filter) {
        if !same_session {
            let _ = bump_epoch();
        }
        return attach_session(canon, filter, index);
    }
    let _ = bump_epoch();
    drop_index();
    let index = build_fresh_index(&dir, &canon, filter)?;
    attach_session(canon, filter, index)
}

pub fn upsert_path(path: &Path, content: &str) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    let filename = filename_of(&path_str);
    with_writer(path, |writer, path_field, filename_field, body_field| {
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

pub fn delete_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    with_writer(path, |writer, path_field, _, _| {
        writer.delete_term(tantivy::Term::from_field_text(path_field, &path_str));
        Ok(())
    })
}

pub fn prune_indices(alive_roots: &[PathBuf]) {
    let parent = indices_parent();
    let Ok(entries) = std::fs::read_dir(&parent) else {
        return;
    };
    let attached = lock_session().ok().and_then(|guard| {
        guard.as_ref().map(|session| workspace_index_id(&session.root))
    });
    let alive: HashSet<String> = alive_roots.iter().map(|root| workspace_index_id(root)).collect();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if attached.as_ref() == Some(&name) {
            continue;
        }
        if !alive.contains(&name) {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

fn hit_in_root(path: &Path, root: Option<&Path>) -> bool {
    let Some(root) = root else {
        return true;
    };
    let canon_root = canonical_root(root);
    path.starts_with(root)
        || path.starts_with(&canon_root)
        || canonical_root(path).starts_with(&canon_root)
}

fn tokens_in(haystacks: &[&str], tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let mut present = HashSet::new();
    for hay in haystacks {
        present.extend(ident_tokens(hay));
    }
    tokens.iter().all(|token| present.contains(token))
}

fn filename_has_token(filename: &str, tokens: &[String]) -> bool {
    let names: HashSet<String> = ident_tokens(filename).into_iter().collect();
    tokens.iter().any(|token| names.contains(token))
}

fn first_token_char_range(line: &str, tokens: &[String]) -> Option<(usize, usize)> {
    let want: HashSet<&str> = tokens.iter().map(String::as_str).collect();
    ident_token_ranges(line)
        .into_iter()
        .filter(|(_, _, token)| want.contains(token.as_str()))
        .map(|(start, len, _)| (start, len))
        .min_by_key(|(start, _)| *start)
}

fn token_snippet(content: &str, tokens: &[String]) -> Option<(String, usize, usize)> {
    if tokens.is_empty() {
        return None;
    }
    for line in content.split('\n') {
        let Some((first, match_len)) = first_token_char_range(line, tokens) else {
            continue;
        };
        let chars: Vec<char> = line.chars().collect();
        let last = first + match_len.saturating_sub(1);
        let start = first.saturating_sub(40);
        let end = (last + 81).min(chars.len());
        let mut snippet = String::new();
        if start > 0 {
            snippet.push('…');
        }
        let window: String = chars[start..end].iter().collect();
        let before_first: String = chars[start..first].iter().collect();
        let match_start = snippet.len() + before_first.len();
        snippet.push_str(&window);
        if end < chars.len() {
            snippet.push('…');
        }
        return Some((snippet, match_start, match_len.max(1)));
    }
    None
}

fn ident_query(filename_field: Field, body_field: Field, tokens: &[String]) -> Option<BooleanQuery> {
    if tokens.is_empty() {
        return None;
    }
    let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    for token in tokens {
        let filename_term = TermQuery::new(
            Term::from_field_text(filename_field, token),
            IndexRecordOption::Basic,
        );
        let body_term = TermQuery::new(
            Term::from_field_text(body_field, token),
            IndexRecordOption::Basic,
        );
        let either = BooleanQuery::new(vec![
            (Occur::Should, Box::new(filename_term) as Box<dyn Query>),
            (Occur::Should, Box::new(body_term) as Box<dyn Query>),
        ]);
        clauses.push((Occur::Must, Box::new(either) as Box<dyn Query>));
    }
    Some(BooleanQuery::new(clauses))
}

fn hit_from_stored(
    path: PathBuf,
    filename: &str,
    body: &str,
    tokens: &[String],
    tantivy_score: f32,
) -> SearchHit {
    let name_match = filename_has_token(filename, tokens);
    let (snippet, match_start, match_len) = token_snippet(body, tokens).unwrap_or_default();
    SearchHit {
        path,
        snippet,
        match_start,
        match_len,
        name_match,
        score: (tantivy_score * 1000.0) as i32,
    }
}

pub fn search(query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    search_in(query, limit, None, current_epoch())
}

fn search_in(
    query: &str,
    limit: usize,
    root: Option<&Path>,
    epoch: u64,
) -> Result<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() || epoch_stale(epoch) {
        return Ok(Vec::new());
    }
    let tokens = ident_tokens(trimmed);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let (index, path_field, filename_field, body_field) = {
        let guard = lock_session()?;
        let session = guard
            .as_ref()
            .ok_or_else(|| eyre!("fts index not initialized"))?;
        (
            session.index.clone(),
            session.path_field,
            session.filename_field,
            session.body_field,
        )
    };

    let Some(tantivy_query) = ident_query(filename_field, body_field, &tokens) else {
        return Ok(Vec::new());
    };

    let reader = index.reader().context("create fts reader")?;
    let searcher = reader.searcher();
    if searcher.num_docs() == 0 {
        return Ok(Vec::new());
    }
    if epoch_stale(epoch) {
        return Ok(Vec::new());
    }

    let fetch = limit.max(1);
    let top_docs = searcher
        .search(&tantivy_query, &TopDocs::with_limit(fetch))
        .context("fts search")?;

    if epoch_stale(epoch) {
        return Ok(Vec::new());
    }

    let mut hits = Vec::new();
    for (score, doc_address) in top_docs {
        if epoch_stale(epoch) {
            return Ok(Vec::new());
        }
        let doc: TantivyDocument = searcher.doc(doc_address).context("load fts doc")?;
        let path_value = doc
            .get_first(path_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| eyre!("fts doc missing path"))?;
        let path = PathBuf::from(path_value);
        if !hit_in_root(&path, root) {
            continue;
        }
        let filename = doc
            .get_first(filename_field)
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| filename_of(path_value));
        let body = doc
            .get_first(body_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        hits.push(hit_from_stored(path, &filename, body, &tokens, score));
    }

    hits.sort_by(|a, b| {
        b.name_match
            .cmp(&a.name_match)
            .then(b.score.cmp(&a.score))
    });
    hits.truncate(limit);
    Ok(hits)
}

fn tab_display_name(tab: &TabItem) -> &str {
    tab.path
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or(tab.title.as_str())
}

fn token_hit_parts(
    name: &str,
    path: &Path,
    content: &str,
    tokens: &[String],
) -> Option<(bool, String, usize, usize, i32)> {
    let path_str = path.to_string_lossy();
    if !tokens_in(&[name, path_str.as_ref(), content], tokens) {
        return None;
    }
    let name_match = filename_has_token(name, tokens) || filename_has_token(path_str.as_ref(), tokens);
    let (snippet, match_start, match_len) = token_snippet(content, tokens)
        .or_else(|| token_snippet(name, tokens))
        .unwrap_or_default();
    Some((name_match, snippet, match_start, match_len, match_len as i32))
}

fn push_or_update_tab_hit(
    hits: &mut Vec<SearchHit>,
    path: PathBuf,
    name: &str,
    content: &str,
    tokens: &[String],
    refresh_snippet: bool,
) {
    let Some((name_match, snippet, match_start, match_len, score)) =
        token_hit_parts(name, &path, content, tokens)
    else {
        return;
    };

    if let Some(existing) = hits.iter_mut().find(|hit| hit.path == path) {
        if refresh_snippet || (existing.snippet.is_empty() && !snippet.is_empty()) {
            existing.snippet = snippet;
            existing.match_start = match_start;
            existing.match_len = match_len;
        }
        existing.name_match |= name_match;
        existing.score = existing.score.max(score);
    } else {
        hits.push(SearchHit {
            path,
            snippet,
            match_start,
            match_len,
            name_match,
            score,
        });
    }
}

pub fn search_all(
    query: &str,
    limit: usize,
    tabs: &[TabItem],
    has_index: bool,
    root: Option<&Path>,
    epoch: u64,
) -> Result<Vec<SearchHit>> {
    if epoch_stale(epoch) {
        return Ok(Vec::new());
    }
    let tokens = ident_tokens(query);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let effective_limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
    let mut hits = if has_index {
        search_in(query, effective_limit.saturating_mul(2), root, epoch)?
    } else {
        Vec::new()
    };
    if epoch_stale(epoch) {
        return Ok(Vec::new());
    }

    for tab in tabs {
        if epoch_stale(epoch) {
            return Ok(Vec::new());
        }
        match &tab.path {
            Some(path) if !tab.is_dirty => {
                if tab.content.is_empty() {
                    continue;
                }
                if let Some(existing) = hits.iter_mut().find(|hit| &hit.path == path) {
                    if let Some((snippet, match_start, match_len)) =
                        token_snippet(&tab.content, &tokens)
                    {
                        existing.snippet = snippet;
                        existing.match_start = match_start;
                        existing.match_len = match_len;
                    }
                }
            }
            Some(path) => {
                if !hit_in_root(path, root) {
                    continue;
                }
                let name = tab_display_name(tab);
                if token_hit_parts(name, path, &tab.content, &tokens).is_none() {
                    hits.retain(|hit| hit.path != *path);
                    continue;
                }
                push_or_update_tab_hit(
                    &mut hits,
                    path.clone(),
                    name,
                    &tab.content,
                    &tokens,
                    true,
                );
            }
            None => {
                if tab.content.is_empty() {
                    continue;
                }
                let path = PathBuf::from(&tab.title);
                push_or_update_tab_hit(
                    &mut hits,
                    path,
                    tab_display_name(tab),
                    &tab.content,
                    &tokens,
                    true,
                );
            }
        }
    }

    hits.sort_by(|a, b| {
        b.name_match
            .cmp(&a.name_match)
            .then(b.score.cmp(&a.score))
    });
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

    struct Case {
        _lock: MutexGuard<'static, ()>,
        dir: PathBuf,
        indices: PathBuf,
    }

    impl Case {
        fn new(name: &str) -> Self {
            let _lock = serial();
            drop_index();
            let dir = temp_dir(name);
            let indices = temp_dir(&format!("{name}_indices"));
            set_indices_root_for_tests(Some(indices.clone()));
            Self {
                _lock,
                dir,
                indices,
            }
        }
    }

    impl Drop for Case {
        fn drop(&mut self) {
            drop_index();
            set_indices_root_for_tests(None);
            let _ = fs::remove_dir_all(&self.dir);
            let _ = fs::remove_dir_all(&self.indices);
        }
    }

    fn sample_tab(path: PathBuf, content: &str, dirty: bool) -> TabItem {
        TabItem {
            id: 1,
            path: Some(path),
            title: "notes.md".to_string(),
            content: content.to_string(),
            parsed: crate::services::markdown::parse_markdown_document(content),
            is_dirty: dirty,
            html_revision: 0,
            parse_gen: 0,
            parse_status: crate::types::ParseStatus::Ready,
        }
    }

    #[test]
    fn test_rebuild_search_upsert_and_drop() {
        let case = Case::new("basic");
        let alpha = case.dir.join("alpha.md");
        let beta = case.dir.join("beta.md");
        fs::write(&alpha, "Hello unique-alpha-token in alpha file.").unwrap();
        fs::write(&beta, "Beta file has other text.").unwrap();

        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        assert!(is_index_for(&case.dir));

        let hits = search("unique-alpha-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        assert!(hits[0].snippet.contains("unique-alpha-token"));
        assert!(hits[0].match_len > 0);

        upsert_path(&alpha, "Hello updated unique-alpha-token content.").unwrap();
        let hits = search("content", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        assert!(search("tokcont", 10).unwrap().is_empty());

        fs::remove_file(&beta).unwrap();
        delete_path(&beta).unwrap();
        let hits = search("Beta", 10).unwrap();
        assert!(hits.is_empty());

        drop_index();
        assert!(!is_index_for(&case.dir));
    }

    #[test]
    fn test_search_filename_without_body_match() {
        let case = Case::new("filename");
        let named = case.dir.join("specialtoken.md");
        fs::write(&named, "nothing relevant in the body at all.").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let hits = search("specialtoken", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, named);
        assert!(hits[0].name_match);
    }

    #[test]
    fn test_search_filename_subsequence_does_not_match() {
        let case = Case::new("filename_subseq");
        let named = case.dir.join("specialtoken.md");
        fs::write(&named, "nothing relevant in the body at all.").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        assert!(search("spctkn", 10).unwrap().is_empty());
        assert_eq!(search("specialtoken", 10).unwrap().len(), 1);
    }

    #[test]
    fn test_search_body_line_subsequence_does_not_match() {
        let case = Case::new("body_subseq");
        let doc = case.dir.join("alpha.md");
        fs::write(&doc, "Hello unique-alpha-token in alpha file.").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        assert!(search("uat", 10).unwrap().is_empty());
        let hits = search("unique", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, doc);
        assert!(hits[0].snippet.contains("unique-alpha-token"));
    }

    #[test]
    fn test_open_existing_index_skips_full_rebuild() {
        let case = Case::new("reopen");
        let alpha = case.dir.join("alpha.md");
        fs::write(&alpha, "keep-me-token lives here.").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        drop_index();
        assert!(!is_index_for(&case.dir));

        fs::write(case.dir.join("extra.md"), "should-not-be-indexed-yet").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        assert!(is_index_for(&case.dir));
        assert!(search("keep-me-token", 10).unwrap().len() == 1);
        assert!(search("should-not-be-indexed-yet", 10).unwrap().is_empty());
    }

    #[test]
    fn test_search_without_drop_keeps_index() {
        let case = Case::new("nodrop");
        let alpha = case.dir.join("alpha.md");
        fs::write(&alpha, "keep-me-token lives here.").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let dir_clone = case.dir.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(5));
            rebuild_root(dir_clone, FileFilterMode::MarkdownOnly)
        });
        let hits = search("keep-me-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        handle.join().unwrap().unwrap();
        assert!(is_index_for(&case.dir));
        let hits = search("keep-me-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_search_all_merges_dirty_tab() {
        let case = Case::new("dirty");
        let doc = case.dir.join("notes.md");
        fs::write(&doc, "disk-only keyword").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let tab = sample_tab(doc.clone(), "dirty-tab unique-ram-token", true);
        let hits = search_all(
            "unique-ram-token",
            10,
            std::slice::from_ref(&tab),
            true,
            Some(&case.dir),
            current_epoch(),
        )
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, doc);
    }

    #[test]
    fn test_search_all_dirty_tab_subsequence_does_not_match() {
        let case = Case::new("dirty_fuzzy");
        let doc = case.dir.join("notes.md");
        fs::write(&doc, "disk-only keyword").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let tab = sample_tab(doc.clone(), "dirty-tab unique-ram-token", true);
        let hits = search_all("urt", 10, std::slice::from_ref(&tab), true, Some(&case.dir), current_epoch())
            .unwrap();
        assert!(hits.is_empty());
        let hits = search_all(
            "unique-ram-token",
            10,
            std::slice::from_ref(&tab),
            true,
            Some(&case.dir),
            current_epoch(),
        )
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.contains("unique-ram-token"));
    }

    #[test]
    fn test_search_all_dirty_tab_without_ram_match_drops_disk_hit() {
        let case = Case::new("dirty_drop");
        let doc = case.dir.join("notes.md");
        fs::write(&doc, "unique-disk-token").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let tab = sample_tab(doc, "totally different ram text", true);
        let hits = search_all(
            "unique-disk-token",
            10,
            std::slice::from_ref(&tab),
            true,
            Some(&case.dir),
            current_epoch(),
        )
        .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn test_search_all_skips_clean_tab_not_in_index() {
        let case = Case::new("cleantab");
        let indexed = case.dir.join("in-index.md");
        fs::write(&indexed, "index-token").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let outside = case.dir.join("not-opened-on-disk.md");
        let tab = sample_tab(outside, "index-token also in this clean tab", false);
        let hits = search_all(
            "index-token",
            10,
            std::slice::from_ref(&tab),
            true,
            Some(&case.dir),
            current_epoch(),
        )
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, indexed);
    }

    #[test]
    fn test_all_files_indexes_non_markdown_content() {
        let case = Case::new("allfiles");
        let rust = case.dir.join("lib.rs");
        fs::write(&rust, "fn unique_rs_token() {}").unwrap();
        fs::write(case.dir.join("notes.md"), "markdown only").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::AllFiles).unwrap();
        assert!(is_full_index_for(&case.dir));

        let hits = search("unique_rs_token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, rust);
        assert!(hits[0].snippet.contains("unique_rs_token"));
    }

    #[test]
    fn test_markdown_only_skips_rust_source() {
        let case = Case::new("mdonly_skip");
        fs::write(case.dir.join("lib.rs"), "fn unique_rs_token() {}").unwrap();
        fs::write(case.dir.join("notes.md"), "markdown only").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        assert!(is_index_for(&case.dir));
        assert!(!is_full_index_for(&case.dir));
        assert!(search("unique_rs_token", 10).unwrap().is_empty());
    }

    #[test]
    fn test_binary_file_does_not_abort_rebuild() {
        let case = Case::new("binary");
        fs::write(case.dir.join("blob.bin"), [0xff, 0xfe, 0x00]).unwrap();
        let doc = case.dir.join("ok.md");
        fs::write(&doc, "ok-token lives here").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::AllFiles).unwrap();

        let hits = search("ok-token", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, doc);

        let name_hits = search("blob", 10).unwrap();
        assert_eq!(name_hits.len(), 1);
        assert!(name_hits[0].name_match);
    }

    #[test]
    fn test_stale_epoch_returns_no_hits() {
        let case = Case::new("stale_epoch");
        fs::write(case.dir.join("notes.md"), "unique-epoch-token").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        let stale = current_epoch();
        bump_epoch();
        let hits = search_all(
            "unique-epoch-token",
            10,
            &[],
            true,
            Some(&case.dir),
            stale,
        )
        .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn test_workspace_meta_sidecar_leaves_tantivy_meta() {
        let case = Case::new("meta_sidecar");
        fs::write(case.dir.join("notes.md"), "hello").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        let id_dir = case.indices.join(workspace_index_id(&case.dir));
        let ours: IndexMeta =
            serde_json::from_str(&fs::read_to_string(id_dir.join(META_FILE_NAME)).unwrap()).unwrap();
        assert_eq!(ours.filter, "md");
        assert_eq!(ours.tokenizer, TOKENIZER_VERSION);
        assert!(tantivy_dir(&id_dir).join(META_FILE_NAME).exists());
    }

    #[test]
    fn test_tokenizer_mismatch_forces_rebuild() {
        let case = Case::new("tok_mismatch");
        fs::write(case.dir.join("old.md"), "keep-me-token").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        drop_index();
        fs::write(case.dir.join("extra.md"), "brand-new-token").unwrap();
        let id_dir = case.indices.join(workspace_index_id(&case.dir));
        let mut meta: IndexMeta =
            serde_json::from_str(&fs::read_to_string(id_dir.join(META_FILE_NAME)).unwrap()).unwrap();
        meta.tokenizer = "old".to_string();
        fs::write(
            id_dir.join(META_FILE_NAME),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();
        assert_eq!(search("brand-new-token", 10).unwrap().len(), 1);
    }

    #[test]
    fn test_prune_indices_removes_evicted_workspace() {
        let case = Case::new("prune");
        let a = case.dir.join("ws_a");
        let b = case.dir.join("ws_b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("a.md"), "token-a").unwrap();
        fs::write(b.join("b.md"), "token-b").unwrap();
        rebuild_root(a.clone(), FileFilterMode::MarkdownOnly).unwrap();
        drop_index();
        rebuild_root(b.clone(), FileFilterMode::MarkdownOnly).unwrap();
        drop_index();

        let parent = case.indices.clone();
        let before = fs::read_dir(&parent).unwrap().count();
        assert_eq!(before, 2);
        prune_indices(std::slice::from_ref(&a));
        let after: Vec<_> = fs::read_dir(&parent)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .collect();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].file_name().to_string_lossy(), workspace_index_id(&a));
    }

    #[test]
    fn test_upsert_and_delete_outside_session_root_are_noop() {
        let case = Case::new("outside_root");
        let inside = case.dir.join("in.md");
        fs::write(&inside, "inside-token").unwrap();
        rebuild_root(case.dir.clone(), FileFilterMode::MarkdownOnly).unwrap();

        let other = temp_dir("outside_root_other");
        let outsider = other.join("out.md");
        fs::write(&outsider, "outside-token").unwrap();
        upsert_path(&outsider, "outside-token").unwrap();
        assert!(search("outside-token", 10).unwrap().is_empty());
        assert_eq!(search("inside-token", 10).unwrap().len(), 1);

        delete_path(&inside).unwrap();
        assert!(search("inside-token", 10).unwrap().is_empty());
        upsert_path(&inside, "inside-token").unwrap();
        delete_path(&outsider).unwrap();
        assert_eq!(search("inside-token", 10).unwrap().len(), 1);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn test_prune_skips_attached_session_dir() {
        let case = Case::new("prune_attached");
        let a = case.dir.join("ws_a");
        let b = case.dir.join("ws_b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("a.md"), "token-a").unwrap();
        fs::write(b.join("b.md"), "token-b").unwrap();
        rebuild_root(a.clone(), FileFilterMode::MarkdownOnly).unwrap();
        drop_index();
        rebuild_root(b.clone(), FileFilterMode::MarkdownOnly).unwrap();

        prune_indices(std::slice::from_ref(&a));
        let names: Vec<_> = fs::read_dir(&case.indices)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&workspace_index_id(&a)));
        assert!(names.contains(&workspace_index_id(&b)));
    }

    #[test]
    fn test_token_spans_marks_whole_idents_only() {
        let marked: String = token_spans("unique-alpha-token", "al")
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, chunk)| chunk)
            .collect();
        assert!(marked.is_empty());
        let marked: String = token_spans("unique-alpha-token", "alpha")
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, chunk)| chunk)
            .collect();
        assert_eq!(marked, "alpha");
        assert!(first_token_char_range("unique-alpha-token", &["al".into()]).is_none());
        assert_eq!(
            first_token_char_range("unique-alpha-token", &["alpha".into()]),
            Some((7, 5))
        );
    }
}
