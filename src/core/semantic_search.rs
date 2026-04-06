use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Result, anyhow, bail};
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use serde::{Deserialize, Serialize};

use super::archive_index::{
    ArchiveIndexEntry, collect_html_archive_entries, resolve_workspace_conversations_dir,
};

const MODEL_ONNX_SUBDIR: &str = "onnx/model.onnx";
const MODEL_ONNX_LEGACY: &str = "model.onnx";
const TOKENIZER_JSON: &str = "tokenizer.json";
const CONFIG_JSON: &str = "config.json";
const SPECIAL_TOKENS_JSON: &str = "special_tokens_map.json";
const TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
const DEFAULT_MODEL_DIR_NAME: &str = "all-MiniLM-L6-v2";
const DEFAULT_EMBEDDER_DIMENSION: usize = 384;
const HYBRID_SEMANTIC_WEIGHT: f32 = 0.7;
const HYBRID_LEXICAL_WEIGHT: f32 = 0.3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticSearchDocument {
    pub entry: ArchiveIndexEntry,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchHit {
    pub entry: ArchiveIndexEntry,
    pub score: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchExecution {
    pub hits: Vec<SemanticSearchHit>,
    pub index_path: PathBuf,
    pub total_documents: usize,
    pub reused_documents: usize,
    pub embedded_documents: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HybridSearchHit {
    pub entry: ArchiveIndexEntry,
    pub hybrid_score: f32,
    pub semantic_score: f32,
    pub lexical_score: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HybridSearchExecution {
    pub hits: Vec<HybridSearchHit>,
    pub index_path: PathBuf,
    pub total_documents: usize,
    pub reused_documents: usize,
    pub embedded_documents: usize,
}

pub trait SemanticEmbedder {
    fn embed_batch_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn is_true_semantic(&self) -> bool;
    fn id(&self) -> &str;
}

pub struct FastEmbedSemanticEmbedder {
    model: Mutex<TextEmbedding>,
    identity: String,
}

impl FastEmbedSemanticEmbedder {
    pub fn default_model_dir() -> Result<PathBuf> {
        let Some(data_dir) = dirs::data_local_dir() else {
            bail!("failed to resolve local data directory for semantic model storage");
        };
        Ok(data_dir
            .join("agent-exporter")
            .join("models")
            .join(DEFAULT_MODEL_DIR_NAME))
    }

    pub fn load_from_dir(model_dir: &Path) -> Result<Self> {
        if !model_dir.is_dir() {
            bail!(
                "semantic retrieval requires local embedding model files at `{}`",
                model_dir.display()
            );
        }

        let model_file = select_model_file(model_dir).ok_or_else(|| {
            anyhow!(
                "missing `{}` or `{}` in `{}`",
                MODEL_ONNX_SUBDIR,
                MODEL_ONNX_LEGACY,
                model_dir.display()
            )
        })?;
        let model_bytes = fs::read(&model_file).map_err(|error| {
            anyhow!(
                "unable to read model file `{}`: {error}",
                model_file.display()
            )
        })?;
        let tokenizer_file = read_required(model_dir.join(TOKENIZER_JSON), TOKENIZER_JSON)?;
        let config_file = read_required(model_dir.join(CONFIG_JSON), CONFIG_JSON)?;
        let special_tokens_map_file =
            read_required(model_dir.join(SPECIAL_TOKENS_JSON), SPECIAL_TOKENS_JSON)?;
        let tokenizer_config_file =
            read_required(model_dir.join(TOKENIZER_CONFIG_JSON), TOKENIZER_CONFIG_JSON)?;
        let identity = build_fastembed_identity(&[
            ("model.onnx", model_bytes.as_slice()),
            (TOKENIZER_JSON, tokenizer_file.as_slice()),
            (CONFIG_JSON, config_file.as_slice()),
            (SPECIAL_TOKENS_JSON, special_tokens_map_file.as_slice()),
            (TOKENIZER_CONFIG_JSON, tokenizer_config_file.as_slice()),
        ]);
        let tokenizer_files = TokenizerFiles {
            tokenizer_file,
            config_file,
            special_tokens_map_file,
            tokenizer_config_file,
        };

        let mut model = UserDefinedEmbeddingModel::new(model_bytes, tokenizer_files);
        model.pooling = Some(Pooling::Mean);

        let init = InitOptionsUserDefined::new();
        let model = TextEmbedding::try_new_from_user_defined(model, init).map_err(|error| {
            anyhow!(
                "failed to initialize fastembed model from `{}`: {error}",
                model_dir.display()
            )
        })?;

        Ok(Self {
            model: Mutex::new(model),
            identity,
        })
    }
}

impl SemanticEmbedder for FastEmbedSemanticEmbedder {
    fn embed_batch_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let mut model = self
            .model
            .lock()
            .map_err(|_| anyhow!("semantic embedder lock poisoned"))?;
        let mut embeddings = model
            .embed(texts.to_vec(), None)
            .map_err(|error| anyhow!("embedding failed: {error}"))?;
        for embedding in embeddings.iter_mut() {
            normalize_in_place(embedding);
            if embedding.len() != DEFAULT_EMBEDDER_DIMENSION {
                bail!(
                    "embedding dimension mismatch: expected {}, got {}",
                    DEFAULT_EMBEDDER_DIMENSION,
                    embedding.len()
                );
            }
        }
        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        DEFAULT_EMBEDDER_DIMENSION
    }

    fn is_true_semantic(&self) -> bool {
        true
    }

    fn id(&self) -> &str {
        &self.identity
    }
}

pub fn collect_semantic_documents(workspace_root: &Path) -> Result<Vec<SemanticSearchDocument>> {
    let archive_dir = resolve_workspace_conversations_dir(workspace_root)?;
    let entries = collect_html_archive_entries(workspace_root)?;
    let mut documents = Vec::with_capacity(entries.len());

    for entry in entries {
        let html_path = archive_dir.join(&entry.relative_href);
        let content = fs::read_to_string(&html_path).map_err(|error| {
            anyhow!(
                "failed to read transcript html `{}`: {error}",
                html_path.display()
            )
        })?;
        let visible_text = html_to_visible_text(&content);
        let text = [
            entry.title.as_str(),
            entry.connector.as_deref().unwrap_or(""),
            entry.thread_id.as_deref().unwrap_or(""),
            entry.completeness.as_deref().unwrap_or(""),
            entry.source_kind.as_deref().unwrap_or(""),
            visible_text.as_str(),
        ]
        .join("\n");
        documents.push(SemanticSearchDocument { entry, text });
    }

    Ok(documents)
}

pub fn semantic_search<E: SemanticEmbedder>(
    embedder: &E,
    documents: &[SemanticSearchDocument],
    query: &str,
    top_k: usize,
) -> Result<Vec<SemanticSearchHit>> {
    let query_embedding = embed_query(embedder, query)?;
    if documents.is_empty() {
        return Ok(Vec::new());
    }

    let corpus_inputs = documents
        .iter()
        .map(|document| document.text.as_str())
        .collect::<Vec<_>>();
    let embeddings = embedder.embed_batch_sync(&corpus_inputs)?;
    if embeddings.len() != documents.len() {
        bail!(
            "embedder returned {} vectors for {} inputs",
            embeddings.len(),
            documents.len()
        );
    }

    rank_hits_from_embedding_refs(
        &query_embedding,
        documents
            .iter()
            .zip(embeddings.iter())
            .map(|(document, embedding)| (&document.entry, embedding.as_slice())),
        top_k,
    )
}

pub fn semantic_search_with_persistent_index<E: SemanticEmbedder>(
    embedder: &E,
    workspace_root: &Path,
    query: &str,
    top_k: usize,
) -> Result<SemanticSearchExecution> {
    let prepared = prepare_persistent_index(embedder, workspace_root)?;
    let query_embedding = embed_query(embedder, query)?;
    let query_hits = rank_semantic_hits_from_index(&query_embedding, &prepared.documents, top_k)?;

    Ok(SemanticSearchExecution {
        hits: query_hits,
        index_path: prepared.index_path,
        total_documents: prepared.total_documents,
        reused_documents: prepared.reused_documents,
        embedded_documents: prepared.embedded_documents,
    })
}

pub fn hybrid_search_with_persistent_index<E: SemanticEmbedder>(
    embedder: &E,
    workspace_root: &Path,
    query: &str,
    top_k: usize,
) -> Result<HybridSearchExecution> {
    let prepared = prepare_persistent_index(embedder, workspace_root)?;
    let query_embedding = embed_query(embedder, query)?;
    let query_hits = rank_hybrid_hits(query, &query_embedding, &prepared.documents, top_k)?;

    Ok(HybridSearchExecution {
        hits: query_hits,
        index_path: prepared.index_path,
        total_documents: prepared.total_documents,
        reused_documents: prepared.reused_documents,
        embedded_documents: prepared.embedded_documents,
    })
}

fn prepare_persistent_index<E: SemanticEmbedder>(
    embedder: &E,
    workspace_root: &Path,
) -> Result<PreparedPersistentIndex> {
    let documents = collect_semantic_documents(workspace_root)?;
    let index_path = resolve_semantic_index_path(workspace_root, embedder.id());
    let mut persisted =
        load_semantic_index(&index_path)?.unwrap_or_else(|| SemanticIndexFile::new(embedder.id()));

    if persisted.embedder != embedder.id() {
        persisted = SemanticIndexFile::new(embedder.id());
    }

    let persisted_map = persisted
        .documents
        .into_iter()
        .map(|document| (document.relative_href.clone(), document))
        .collect::<HashMap<_, _>>();

    let mut reused_documents = 0usize;
    let mut embedded_documents = 0usize;
    let mut indexed_documents = documents
        .iter()
        .map(|document| IndexedSemanticDocument {
            entry: document.entry.clone(),
            text: document.text.clone(),
            embedding: None,
        })
        .collect::<Vec<_>>();
    let mut pending_slots = Vec::new();
    let mut pending_texts = Vec::new();

    for (index, document) in documents.iter().enumerate() {
        if let Some(existing) = persisted_map.get(&document.entry.relative_href) {
            if existing.text == document.text {
                indexed_documents[index].embedding = Some(existing.embedding.clone());
                reused_documents += 1;
                continue;
            }
        }
        pending_slots.push(index);
        pending_texts.push(document.text.as_str());
    }

    if !pending_texts.is_empty() {
        let vectors = embedder.embed_batch_sync(&pending_texts)?;
        if vectors.len() != pending_slots.len() {
            bail!(
                "embedder returned {} vectors for {} pending documents",
                vectors.len(),
                pending_slots.len()
            );
        }
        for (slot, vector) in pending_slots.into_iter().zip(vectors.into_iter()) {
            indexed_documents[slot].embedding = Some(vector);
            embedded_documents += 1;
        }
    }

    let index_file = SemanticIndexFile {
        schema_version: 1,
        embedder: embedder.id().to_string(),
        documents: indexed_documents
            .iter()
            .map(|document| PersistedSemanticDocument {
                relative_href: document.entry.relative_href.clone(),
                entry: document.entry.clone(),
                text: document.text.clone(),
                embedding: document.embedding.clone().unwrap_or_default(),
            })
            .collect(),
    };
    write_semantic_index(&index_path, &index_file)?;

    Ok(PreparedPersistentIndex {
        documents: indexed_documents,
        index_path,
        total_documents: documents.len(),
        reused_documents,
        embedded_documents,
    })
}

fn select_model_file(model_dir: &Path) -> Option<PathBuf> {
    let modern = model_dir.join(MODEL_ONNX_SUBDIR);
    if modern.is_file() {
        return Some(modern);
    }
    let legacy = model_dir.join(MODEL_ONNX_LEGACY);
    if legacy.is_file() {
        return Some(legacy);
    }
    None
}

fn embed_query<E: SemanticEmbedder>(embedder: &E, query: &str) -> Result<Vec<f32>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("semantic search requires --query <TEXT>");
    }
    if !embedder.is_true_semantic() {
        bail!("semantic search requires a true semantic embedder");
    }

    let embeddings = embedder.embed_batch_sync(&[query])?;
    if embeddings.len() != 1 {
        bail!("embedder returned {} vectors for 1 input", embeddings.len());
    }
    let embedding = embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("missing query embedding"))?;
    if embedding.len() != embedder.dimension() {
        bail!(
            "embedding dimension mismatch: expected {}, got {}",
            embedder.dimension(),
            embedding.len()
        );
    }
    Ok(embedding)
}

fn rank_hits_from_embedding_refs<'a, I>(
    query_embedding: &[f32],
    documents: I,
    top_k: usize,
) -> Result<Vec<SemanticSearchHit>>
where
    I: IntoIterator<Item = (&'a ArchiveIndexEntry, &'a [f32])>,
{
    let mut hits = Vec::new();

    for (entry, embedding) in documents {
        if embedding.len() != query_embedding.len() {
            bail!(
                "embedding dimension mismatch: expected {}, got {}",
                query_embedding.len(),
                embedding.len()
            );
        }
        hits.push(SemanticSearchHit {
            entry: entry.clone(),
            score: cosine_similarity(query_embedding, embedding),
        });
    }

    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.entry.file_name.cmp(&right.entry.file_name))
    });
    hits.truncate(top_k.max(1));
    Ok(hits)
}

fn rank_semantic_hits_from_index(
    query_embedding: &[f32],
    documents: &[IndexedSemanticDocument],
    top_k: usize,
) -> Result<Vec<SemanticSearchHit>> {
    let mut hits = Vec::with_capacity(documents.len());

    for document in documents {
        let embedding = document.embedding.as_deref().ok_or_else(|| {
            anyhow!(
                "missing persisted embedding for `{}`",
                document.entry.relative_href
            )
        })?;
        if embedding.len() != query_embedding.len() {
            bail!(
                "embedding dimension mismatch: expected {}, got {}",
                query_embedding.len(),
                embedding.len()
            );
        }
        hits.push(SemanticSearchHit {
            entry: document.entry.clone(),
            score: cosine_similarity(query_embedding, embedding),
        });
    }

    sort_semantic_hits(&mut hits, top_k);
    Ok(hits)
}

fn rank_hybrid_hits(
    query: &str,
    query_embedding: &[f32],
    documents: &[IndexedSemanticDocument],
    top_k: usize,
) -> Result<Vec<HybridSearchHit>> {
    let mut hits = Vec::with_capacity(documents.len());

    for document in documents {
        let embedding = document.embedding.as_deref().ok_or_else(|| {
            anyhow!(
                "missing persisted embedding for `{}`",
                document.entry.relative_href
            )
        })?;
        if embedding.len() != query_embedding.len() {
            bail!(
                "embedding dimension mismatch: expected {}, got {}",
                query_embedding.len(),
                embedding.len()
            );
        }

        let semantic_score = cosine_similarity(query_embedding, embedding);
        let lexical_score = lexical_metadata_score(&document.entry, query);
        let hybrid_score = blend_hybrid_score(semantic_score, lexical_score);

        hits.push(HybridSearchHit {
            entry: document.entry.clone(),
            hybrid_score,
            semantic_score,
            lexical_score,
        });
    }

    hits.sort_by(|left, right| {
        right
            .hybrid_score
            .partial_cmp(&left.hybrid_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.entry.file_name.cmp(&right.entry.file_name))
    });
    hits.truncate(top_k.max(1));
    Ok(hits)
}

fn lexical_metadata_score(entry: &ArchiveIndexEntry, query: &str) -> f32 {
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        return 0.0;
    }

    let searchable = [
        entry.title.as_str(),
        entry.connector.as_deref().unwrap_or(""),
        entry.thread_id.as_deref().unwrap_or(""),
        entry.completeness.as_deref().unwrap_or(""),
        entry.source_kind.as_deref().unwrap_or(""),
        entry.file_name.as_str(),
    ]
    .join(" ")
    .to_lowercase();
    let tokens = normalized_query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        return 0.0;
    }

    let matched = tokens
        .iter()
        .filter(|token| searchable.contains(**token))
        .count() as f32;
    let mut score = matched / tokens.len() as f32;
    if searchable.contains(&normalized_query) {
        score += 0.25;
    }
    score.clamp(0.0, 1.0)
}

fn blend_hybrid_score(semantic_score: f32, lexical_score: f32) -> f32 {
    let semantic_component = ((semantic_score + 1.0) / 2.0).clamp(0.0, 1.0);
    (semantic_component * HYBRID_SEMANTIC_WEIGHT) + (lexical_score * HYBRID_LEXICAL_WEIGHT)
}

fn read_required(path: PathBuf, label: &str) -> Result<Vec<u8>> {
    fs::read(&path)
        .map_err(|error| anyhow!("unable to read {label} at `{}`: {error}", path.display()))
}

fn build_fastembed_identity(files: &[(&str, &[u8])]) -> String {
    format!(
        "fastembed-minilm-384-{:016x}",
        stable_asset_fingerprint(files)
    )
}

fn stable_asset_fingerprint(files: &[(&str, &[u8])]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for (label, bytes) in files {
        hash = update_fingerprint(hash, label.as_bytes(), PRIME);
        hash = update_fingerprint(hash, &[0], PRIME);
        hash = update_fingerprint(hash, bytes, PRIME);
        hash = update_fingerprint(hash, &[0xff], PRIME);
    }
    hash
}

fn update_fingerprint(mut hash: u64, bytes: &[u8], prime: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}

fn normalize_in_place(embedding: &mut [f32]) {
    let norm_sq: f32 = embedding.iter().map(|x| x * x).sum();
    if norm_sq.is_finite() && norm_sq > f32::EPSILON {
        let inv_norm = 1.0 / norm_sq.sqrt();
        for value in embedding.iter_mut() {
            *value *= inv_norm;
        }
    } else {
        embedding.fill(0.0);
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(a, b)| a * b).sum()
}

fn sort_semantic_hits(hits: &mut Vec<SemanticSearchHit>, top_k: usize) {
    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.entry.file_name.cmp(&right.entry.file_name))
    });
    hits.truncate(top_k.max(1));
}

fn html_to_visible_text(html: &str) -> String {
    let without_style = strip_tag_block(html, "style");
    let without_script = strip_tag_block(&without_style, "script");
    let mut output = String::with_capacity(without_script.len());
    let mut in_tag = false;
    for ch in without_script.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    collapse_whitespace(&unescape_html(&output))
}

fn strip_tag_block(html: &str, tag_name: &str) -> String {
    let open_prefix = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut remaining = html;
    let mut output = String::new();

    loop {
        let Some(start) = remaining.to_lowercase().find(&open_prefix) else {
            output.push_str(remaining);
            break;
        };
        output.push_str(&remaining[..start]);
        let tail = &remaining[start..];
        let lower_tail = tail.to_lowercase();
        let Some(end) = lower_tail.find(&close_tag) else {
            break;
        };
        let end_index = end + close_tag.len();
        remaining = &tail[end_index..];
    }

    output
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn unescape_html(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn resolve_semantic_index_path(workspace_root: &Path, embedder_id: &str) -> PathBuf {
    workspace_root
        .join(".agents")
        .join("Search")
        .join(format!("semantic-index-{embedder_id}.json"))
}

fn load_semantic_index(path: &Path) -> Result<Option<SemanticIndexFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|error| {
        anyhow!(
            "failed to read semantic index `{}`: {error}",
            path.display()
        )
    })?;
    let index = serde_json::from_str(&content).map_err(|error| {
        anyhow!(
            "failed to parse semantic index `{}`: {error}",
            path.display()
        )
    })?;
    Ok(Some(index))
}

fn write_semantic_index(path: &Path, index: &SemanticIndexFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            anyhow!(
                "failed to prepare semantic index directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    let rendered = serde_json::to_string_pretty(index)
        .map_err(|error| anyhow!("failed to render semantic index json: {error}"))?;
    fs::write(path, format!("{rendered}\n")).map_err(|error| {
        anyhow!(
            "failed to write semantic index `{}`: {error}",
            path.display()
        )
    })
}

#[derive(Clone, Debug)]
struct IndexedSemanticDocument {
    entry: ArchiveIndexEntry,
    text: String,
    embedding: Option<Vec<f32>>,
}

#[derive(Clone, Debug)]
struct PreparedPersistentIndex {
    documents: Vec<IndexedSemanticDocument>,
    index_path: PathBuf,
    total_documents: usize,
    reused_documents: usize,
    embedded_documents: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SemanticIndexFile {
    schema_version: u32,
    embedder: String,
    documents: Vec<PersistedSemanticDocument>,
}

impl SemanticIndexFile {
    fn new(embedder: &str) -> Self {
        Self {
            schema_version: 1,
            embedder: embedder.to_string(),
            documents: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedSemanticDocument {
    relative_href: String,
    entry: ArchiveIndexEntry,
    text: String,
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use anyhow::Result;
    use tempfile::tempdir;

    use super::{
        CONFIG_JSON, SPECIAL_TOKENS_JSON, SemanticEmbedder, TOKENIZER_CONFIG_JSON, TOKENIZER_JSON,
        build_fastembed_identity, collect_semantic_documents, hybrid_search_with_persistent_index,
        semantic_search, semantic_search_with_persistent_index,
    };

    struct MockSemanticEmbedder;

    impl SemanticEmbedder for MockSemanticEmbedder {
        fn embed_batch_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .map(|text| {
                    let lower = text.to_lowercase();
                    if lower.contains("login") || lower.contains("auth") {
                        vec![1.0, 0.0, 0.0]
                    } else if lower.contains("payment") || lower.contains("billing") {
                        vec![0.0, 1.0, 0.0]
                    } else {
                        vec![0.0, 0.0, 1.0]
                    }
                })
                .collect())
        }

        fn dimension(&self) -> usize {
            3
        }

        fn is_true_semantic(&self) -> bool {
            true
        }

        fn id(&self) -> &str {
            "mock-semantic"
        }
    }

    struct CountingSemanticEmbedder {
        id: &'static str,
        calls: Mutex<Vec<Vec<String>>>,
    }

    impl CountingSemanticEmbedder {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                calls: Mutex::new(Vec::new()),
            }
        }

        fn take_calls(&self) -> Vec<Vec<String>> {
            let mut calls = self.calls.lock().expect("calls lock");
            std::mem::take(&mut *calls)
        }
    }

    impl SemanticEmbedder for CountingSemanticEmbedder {
        fn embed_batch_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(texts.iter().map(|text| (*text).to_string()).collect());
            MockSemanticEmbedder.embed_batch_sync(texts)
        }

        fn dimension(&self) -> usize {
            3
        }

        fn is_true_semantic(&self) -> bool {
            true
        }

        fn id(&self) -> &str {
            self.id
        }
    }

    #[test]
    fn collect_semantic_documents_extracts_visible_text() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&archive_dir).expect("mkdirs");
        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<title>demo transcript</title>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Demo transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "<style>.hidden { display:none; }</style>",
                "</head><body><h1>Login issue</h1><p>Auth flow broken</p></body></html>"
            ),
        )
        .expect("write transcript");

        let docs = collect_semantic_documents(workspace.path()).expect("collect docs");
        assert_eq!(docs.len(), 1);
        assert!(docs[0].text.contains("Login issue"));
        assert!(docs[0].text.contains("Auth flow broken"));
        assert!(!docs[0].text.contains("display:none"));
    }

    #[test]
    fn semantic_search_ranks_semantically_closest_document_first() {
        let docs = vec![
            super::SemanticSearchDocument {
                entry: crate::core::archive_index::ArchiveIndexEntry {
                    file_name: "auth.html".to_string(),
                    relative_href: "auth.html".to_string(),
                    title: "Auth transcript".to_string(),
                    connector: Some("codex".to_string()),
                    thread_id: Some("auth-thread".to_string()),
                    completeness: Some("complete".to_string()),
                    source_kind: Some("app-server-thread-read".to_string()),
                    exported_at: Some("2026-04-05T00:00:00Z".to_string()),
                },
                text: "login auth bug".to_string(),
            },
            super::SemanticSearchDocument {
                entry: crate::core::archive_index::ArchiveIndexEntry {
                    file_name: "billing.html".to_string(),
                    relative_href: "billing.html".to_string(),
                    title: "Billing transcript".to_string(),
                    connector: Some("claude-code".to_string()),
                    thread_id: Some("billing-thread".to_string()),
                    completeness: Some("degraded".to_string()),
                    source_kind: Some("claude-session-path".to_string()),
                    exported_at: Some("2026-04-05T00:00:00Z".to_string()),
                },
                text: "payment bug".to_string(),
            },
        ];

        let hits = semantic_search(
            &MockSemanticEmbedder,
            &docs,
            "How do I fix login problems?",
            5,
        )
        .expect("semantic hits");

        assert_eq!(
            hits.first().expect("first hit").entry.file_name,
            "auth.html"
        );
        assert!(hits[0].score >= hits[1].score);
    }

    #[test]
    fn semantic_search_with_persistent_index_reuses_existing_embeddings() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&archive_dir).expect("mkdirs");
        std::fs::write(
            archive_dir.join("auth.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Auth transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"auth-thread\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>login auth bug</p></body></html>"
            ),
        )
        .expect("write auth");

        let embedder = CountingSemanticEmbedder::new("mock-semantic");
        let first =
            semantic_search_with_persistent_index(&embedder, workspace.path(), "login issue", 5)
                .expect("first search");
        let first_calls = embedder.take_calls();
        let second =
            semantic_search_with_persistent_index(&embedder, workspace.path(), "login issue", 5)
                .expect("second search");
        let second_calls = embedder.take_calls();

        assert_eq!(first.embedded_documents, 1);
        assert_eq!(second.reused_documents, 1);
        assert_eq!(second.embedded_documents, 0);
        assert!(second.index_path.exists());
        assert_eq!(first_calls.len(), 2);
        assert_eq!(first_calls[0].len(), 1);
        assert_eq!(first_calls[1], vec!["login issue".to_string()]);
        assert_eq!(second_calls, vec![vec!["login issue".to_string()]]);
    }

    #[test]
    fn semantic_search_with_persistent_index_does_not_reuse_other_embedder_identity() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&archive_dir).expect("mkdirs");
        std::fs::write(
            archive_dir.join("auth.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Auth transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"auth-thread\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>login auth bug</p></body></html>"
            ),
        )
        .expect("write auth");

        let first = semantic_search_with_persistent_index(
            &CountingSemanticEmbedder::new("mock-semantic-a"),
            workspace.path(),
            "login issue",
            5,
        )
        .expect("first search");
        let second = semantic_search_with_persistent_index(
            &CountingSemanticEmbedder::new("mock-semantic-b"),
            workspace.path(),
            "login issue",
            5,
        )
        .expect("second search");

        assert_ne!(first.index_path, second.index_path);
        assert_eq!(second.reused_documents, 0);
        assert_eq!(second.embedded_documents, 1);
    }

    #[test]
    fn hybrid_search_uses_metadata_signal_without_mutating_semantic_path() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        std::fs::create_dir_all(&archive_dir).expect("mkdirs");
        std::fs::write(
            archive_dir.join("thread-1.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"General transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>misc notes only</p></body></html>"
            ),
        )
        .expect("write thread-1");
        std::fs::write(
            archive_dir.join("thread-2.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"General transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-2\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>misc notes only</p></body></html>"
            ),
        )
        .expect("write thread-2");

        let hits = hybrid_search_with_persistent_index(
            &CountingSemanticEmbedder::new("mock-semantic-hybrid"),
            workspace.path(),
            "thread-1",
            5,
        )
        .expect("hybrid hits");

        assert_eq!(
            hits.hits.first().expect("first hit").entry.file_name,
            "thread-1.html"
        );
        assert!(
            hits.hits.first().expect("first hit").lexical_score
                > hits.hits.get(1).expect("second hit").lexical_score
        );
    }

    #[test]
    fn fastembed_identity_changes_when_asset_bytes_change() {
        let first = build_fastembed_identity(&[
            ("model.onnx", &[1, 2, 3]),
            (TOKENIZER_JSON, &[4, 5]),
            (CONFIG_JSON, &[6]),
            (SPECIAL_TOKENS_JSON, &[7]),
            (TOKENIZER_CONFIG_JSON, &[8]),
        ]);
        let second = build_fastembed_identity(&[
            ("model.onnx", &[1, 2, 4]),
            (TOKENIZER_JSON, &[4, 5]),
            (CONFIG_JSON, &[6]),
            (SPECIAL_TOKENS_JSON, &[7]),
            (TOKENIZER_CONFIG_JSON, &[8]),
        ]);

        assert_ne!(first, second);
    }

    #[test]
    fn collect_semantic_documents_ignores_search_reports_directory() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        let reports_dir = workspace
            .path()
            .join(".agents")
            .join("Search")
            .join("Reports");
        std::fs::create_dir_all(&archive_dir).expect("archive mkdirs");
        std::fs::create_dir_all(&reports_dir).expect("reports mkdirs");

        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Demo transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>real transcript</p></body></html>"
            ),
        )
        .expect("write transcript");
        std::fs::write(
            reports_dir.join("search-report-semantic.html"),
            "<!DOCTYPE html><html><head><title>report</title></head><body><p>report should not become corpus</p></body></html>",
        )
        .expect("write report");

        let documents = collect_semantic_documents(workspace.path()).expect("collect docs");
        assert_eq!(documents.len(), 1);
        assert!(documents[0].text.contains("real transcript"));
        assert!(
            !documents[0]
                .text
                .contains("report should not become corpus")
        );
    }

    #[test]
    fn collect_semantic_documents_ignores_integration_reports_directory() {
        let workspace = tempdir().expect("workspace");
        let archive_dir = workspace.path().join(".agents").join("Conversations");
        let integration_reports_dir = workspace
            .path()
            .join(".agents")
            .join("Integration")
            .join("Reports");
        std::fs::create_dir_all(&archive_dir).expect("archive mkdirs");
        std::fs::create_dir_all(&integration_reports_dir).expect("integration mkdirs");

        std::fs::write(
            archive_dir.join("demo.html"),
            concat!(
                "<!DOCTYPE html><html><head>",
                "<meta name=\"agent-exporter:thread-display-name\" content=\"Demo transcript\">",
                "<meta name=\"agent-exporter:connector\" content=\"codex\">",
                "<meta name=\"agent-exporter:thread-id\" content=\"thread-1\">",
                "<meta name=\"agent-exporter:completeness\" content=\"complete\">",
                "<meta name=\"agent-exporter:source-kind\" content=\"app-server-thread-read\">",
                "<meta name=\"agent-exporter:exported-at\" content=\"2026-04-05T00:00:00Z\">",
                "</head><body><p>real transcript</p></body></html>"
            ),
        )
        .expect("write transcript");
        std::fs::write(
            integration_reports_dir.join("integration-report-onboard-codex.html"),
            "<!DOCTYPE html><html><head><title>integration report</title></head><body><p>integration evidence should not become corpus</p></body></html>",
        )
        .expect("write integration report");
        std::fs::write(
            integration_reports_dir.join("integration-report-onboard-codex.json"),
            "{\"summary\":\"integration evidence should not become corpus\"}",
        )
        .expect("write integration report json");
        std::fs::write(
            integration_reports_dir.join("index.json"),
            "{\"report_count\":1}",
        )
        .expect("write integration report index json");

        let documents = collect_semantic_documents(workspace.path()).expect("collect docs");
        assert_eq!(documents.len(), 1);
        assert!(documents[0].text.contains("real transcript"));
        assert!(
            !documents[0]
                .text
                .contains("integration evidence should not become corpus")
        );
    }
}
