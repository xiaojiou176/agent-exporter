use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Result, anyhow, bail};
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};

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

pub trait SemanticEmbedder {
    fn embed_batch_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn is_true_semantic(&self) -> bool;
}

pub struct FastEmbedSemanticEmbedder {
    model: Mutex<TextEmbedding>,
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
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: read_required(model_dir.join(TOKENIZER_JSON), TOKENIZER_JSON)?,
            config_file: read_required(model_dir.join(CONFIG_JSON), CONFIG_JSON)?,
            special_tokens_map_file: read_required(
                model_dir.join(SPECIAL_TOKENS_JSON),
                SPECIAL_TOKENS_JSON,
            )?,
            tokenizer_config_file: read_required(
                model_dir.join(TOKENIZER_CONFIG_JSON),
                TOKENIZER_CONFIG_JSON,
            )?,
        };

        let mut model = UserDefinedEmbeddingModel::new(
            fs::read(&model_file).map_err(|error| {
                anyhow!(
                    "unable to read model file `{}`: {error}",
                    model_file.display()
                )
            })?,
            tokenizer_files,
        );
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
    let query = query.trim();
    if query.is_empty() {
        bail!("semantic search requires --query <TEXT>");
    }
    if !embedder.is_true_semantic() {
        bail!("semantic search requires a true semantic embedder");
    }
    if documents.is_empty() {
        return Ok(Vec::new());
    }

    let mut corpus_inputs = Vec::with_capacity(documents.len() + 1);
    corpus_inputs.push(query);
    for document in documents {
        corpus_inputs.push(document.text.as_str());
    }
    let embeddings = embedder.embed_batch_sync(&corpus_inputs)?;
    if embeddings.len() != documents.len() + 1 {
        bail!(
            "embedder returned {} vectors for {} inputs",
            embeddings.len(),
            documents.len() + 1
        );
    }

    let query_embedding = &embeddings[0];
    let mut hits = documents
        .iter()
        .zip(embeddings.iter().skip(1))
        .map(|(document, embedding)| SemanticSearchHit {
            entry: document.entry.clone(),
            score: cosine_similarity(query_embedding, embedding),
        })
        .collect::<Vec<_>>();

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

fn read_required(path: PathBuf, label: &str) -> Result<Vec<u8>> {
    fs::read(&path)
        .map_err(|error| anyhow!("unable to read {label} at `{}`: {error}", path.display()))
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::tempdir;

    use super::{SemanticEmbedder, collect_semantic_documents, semantic_search};

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
}
