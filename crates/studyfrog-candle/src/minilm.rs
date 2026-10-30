//! The semantic runtime: a MiniLM-class sentence embedder (BERT encoder, mean pooling, unit
//! length) run by candle on the CPU. The model files are read from a directory the user provides;
//! nothing is bundled (ADR 0004).
//!
//! Needs `config.json`, `tokenizer.json` and `model.safetensors` of a BERT sentence-embedding model
//! such as `sentence-transformers/all-MiniLM-L6-v2`.

use std::path::Path;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::{PaddingParams, Tokenizer, TruncationParams};

use studyfrog_slm::runtime::{SlmError, SlmRuntime};
use studyfrog_slm::vector::normalize;

/// MiniLM rates acceptable answers from about 0.65 up; the best pivot on the labelled set is 0.67 and
/// 0.7 is nearly as good (26 against 27 of 33), so no stretching is needed.
pub const MINILM_PIVOT: f32 = 0.7;

/// Longest input in tokens; longer texts are cut (answers are short).
pub const MAX_TOKENS: usize = 256;

/// A loaded model. Plain data; behaviour is in the free functions of this module.
pub struct Minilm {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

fn failed(what: &str, e: impl std::fmt::Display) -> SlmError {
    SlmError::Failed(format!("{what}: {e}"))
}

fn unavailable(path: &Path, e: impl std::fmt::Display) -> SlmError {
    SlmError::Unavailable(format!("{}: {e}", path.display()))
}

/// Load the model files from `dir`.
pub fn minilm_load(dir: &Path) -> Result<Minilm, SlmError> {
    let read =
        |name: &str| std::fs::read(dir.join(name)).map_err(|e| unavailable(&dir.join(name), e));
    let config: Config = serde_json::from_slice(&read("config.json")?)
        .map_err(|e| failed("config.json is not a BERT configuration", e))?;
    let mut tokenizer = Tokenizer::from_file(dir.join("tokenizer.json"))
        .map_err(|e| unavailable(&dir.join("tokenizer.json"), e))?;
    tokenizer
        .with_truncation(Some(TruncationParams {
            max_length: MAX_TOKENS,
            ..Default::default()
        }))
        .map_err(|e| failed("tokenizer", e))?;
    tokenizer.with_padding(Some(PaddingParams::default()));
    let device = Device::Cpu;
    let weights =
        VarBuilder::from_buffered_safetensors(read("model.safetensors")?, DType::F32, &device)
            .map_err(|e| failed("model.safetensors", e))?;
    let model = BertModel::load(weights, &config).map_err(|e| failed("loading the model", e))?;
    Ok(Minilm {
        model,
        tokenizer,
        device,
    })
}

fn tensor(rows: Vec<Vec<u32>>, device: &Device) -> candle_core::Result<Tensor> {
    let (n, len) = (rows.len(), rows.first().map_or(0, Vec::len));
    Tensor::from_vec(rows.concat(), (n, len), device)
}

/// Embed the texts: one unit-length vector each.
pub fn minilm_embed(m: &Minilm, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let encodings = m
        .tokenizer
        .encode_batch(texts.to_vec(), true)
        .map_err(|e| failed("tokenizing", e))?;
    let ids: Vec<Vec<u32>> = encodings.iter().map(|e| e.get_ids().to_vec()).collect();
    let mask: Vec<Vec<u32>> = encodings
        .iter()
        .map(|e| e.get_attention_mask().to_vec())
        .collect();
    pooled(m, ids, mask).map_err(|e| failed("running the model", e))
}

fn pooled(
    m: &Minilm,
    ids: Vec<Vec<u32>>,
    mask: Vec<Vec<u32>>,
) -> candle_core::Result<Vec<Vec<f32>>> {
    let ids = tensor(ids, &m.device)?;
    let mask = tensor(mask, &m.device)?;
    let kinds = ids.zeros_like()?;
    let hidden = m.model.forward(&ids, &kinds, Some(&mask))?;
    let weights = mask.to_dtype(DType::F32)?.unsqueeze(2)?;
    let sum = hidden.broadcast_mul(&weights)?.sum(1)?;
    let count = weights.sum(1)?.clamp(1e-9, f64::MAX)?;
    let mut rows = sum.broadcast_div(&count)?.to_vec2::<f32>()?;
    rows.iter_mut().for_each(|r| normalize(r));
    Ok(rows)
}

impl SlmRuntime for Minilm {
    fn name(&self) -> &str {
        "language model (MiniLM)"
    }

    fn pivot(&self) -> f32 {
        MINILM_PIVOT
    }

    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError> {
        minilm_embed(self, texts)
    }
}
