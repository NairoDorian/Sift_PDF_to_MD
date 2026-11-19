//! surya 2.5 on llama-server (V0 `sift/surya/client.py`, from surya's backends/llamacpp.py
//! and openai_client.py): flags, request format, retries. The vendor's sampling is kept
//! (temperature 0, top-p 0.1, retries warmer: D21). Every request goes through the answer
//! store first (5.4): a stored answer is reused, a live one is stored.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::answers::{AnswerStore, request_key};
use crate::imaging::{Rgb, scale_to_fit};
use crate::llm::{Completion, LlamaServer, StateFn, chat, detect_repeat_token, image_message};

pub const CTX_PER_SLOT: usize = 12288;
const END_MARKERS: [&str; 3] = ["<|im_end!>", "<|im_end|>", "<|endoftext|>"];

pub fn strip_end_marker(text: &str) -> String {
    let cut = END_MARKERS.iter().filter_map(|m| text.find(m)).min();
    match cut {
        Some(i) => crate::py::rstrip(&text[..i]).to_string(),
        None => text.to_string(),
    }
}

pub fn server_args(model: &Path, mmproj: &Path, parallel: usize, ctx_per_slot: usize) -> Vec<String> {
    let ctx = 16384.max(parallel * ctx_per_slot);
    vec![
        "-m".into(),
        model.display().to_string(),
        "--mmproj".into(),
        mmproj.display().to_string(),
        "-ngl".into(),
        "99".into(),
        "--parallel".into(),
        parallel.to_string(),
        "--ctx-size".into(),
        ctx.to_string(),
        "--alias".into(),
        "surya".into(),
        "--jinja".into(),
        "--special".into(),
    ]
}

pub fn make_server(
    model: &Path,
    mmproj: &Path,
    parallel: usize,
    ctx_per_slot: usize,
    build: &Path,
    log_dir: &Path,
    on_state: StateFn,
) -> LlamaServer {
    LlamaServer::new("surya", server_args(model, mmproj, parallel, ctx_per_slot), build, log_dir, on_state)
}

/// A request image: scale_to_fit (28-px grid) once at crop time, plus its PNG.
#[derive(Clone)]
pub struct Prepared {
    pub rgb: Arc<Rgb>,
    pub png: Arc<Vec<u8>>,
}

pub fn prepare_image(img: &Rgb) -> Prepared {
    let s = scale_to_fit(img);
    let png = s.png(true);
    Prepared { rgb: Arc::new(s), png: Arc::new(png) }
}

fn once(server: &LlamaServer, png: &[u8], prompt: &str, max_tokens: u32, temperature: f64, top_p: f64) -> Completion {
    let mut c = chat(
        server,
        image_message(png, prompt),
        max_tokens,
        &[("temperature", json!(temperature)), ("top_p", json!(top_p))],
    );
    c.text = strip_end_marker(&c.text);
    c
}

fn should_retry(c: &Completion, retries: u32, max_retries: u32) -> bool {
    if retries >= max_retries {
        return false;
    }
    if c.error.is_some() {
        return true;
    }
    detect_repeat_token(&c.text, 0) || (c.text.chars().count() > 50 && detect_repeat_token(&c.text, 50))
}

pub fn answer_to_value(c: &Completion) -> Value {
    json!({"text": c.text, "finish_reason": c.finish_reason, "tokens": c.tokens, "retries": c.retries, "error": c.error})
}

pub fn value_to_answer(v: &Value) -> Completion {
    Completion {
        text: strip_end_marker(v["text"].as_str().unwrap_or("")),
        finish_reason: v["finish_reason"].as_str().map(String::from),
        tokens: v["tokens"].as_u64().unwrap_or(0),
        retries: v["retries"].as_u64().unwrap_or(0) as u32,
        error: v["error"].as_str().map(String::from),
        cached: true,
        ..Default::default()
    }
}

/// surya's chat_completions_batch retry loop for one item, behind the answer store.
pub fn generate(
    server: Option<&LlamaServer>,
    store: &AnswerStore,
    img: &Prepared,
    prompt: &str,
    max_tokens: u32,
) -> Completion {
    let key = request_key("surya", prompt, &[img.rgb.pixel_hash()], max_tokens, "");
    if let Some(v) = store.get(&key) {
        return value_to_answer(&v);
    }
    let Some(server) = server.filter(|_| !store.replay_only) else {
        store.note_miss(format!("surya {max_tokens} {}x{}", img.rgb.width, img.rgb.height));
        return Completion {
            error: Some(if store.replay_only {
                "not in the answer store (replay only)".into()
            } else {
                "surya server unavailable".into()
            }),
            ..Default::default()
        };
    };
    let (temperature, top_p, max_retries) = (0.0, 0.1, 3);
    let mut c = once(server, &img.png, prompt, max_tokens, temperature, top_p);
    let mut retries = 0;
    while should_retry(&c, retries, max_retries) {
        if server.is_stopped() {
            break;
        }
        if c.error.is_some() {
            std::thread::sleep(std::time::Duration::from_millis((1500.0 * (retries as f64 + 1.0)) as u64));
        }
        let t = (temperature + 0.2 * (retries as f64 + 1.0)).min(0.8);
        let p = if c.error.is_none() { 0.95 } else { top_p };
        c = once(server, &img.png, prompt, max_tokens, t, p);
        retries += 1;
    }
    c.retries = retries;
    if c.error.is_none() {
        store.put(&key, "surya", prompt, max_tokens, "", &answer_to_value(&c));
    }
    c
}

pub fn load_prompt(name: &str) -> String {
    let p = crate::paths::prompts_dir().join("surya-2.5").join(format!("{name}.txt"));
    crate::paths::read_text(&p).unwrap_or_else(|e| panic!("surya prompt {}: {e}", p.display()))
}
