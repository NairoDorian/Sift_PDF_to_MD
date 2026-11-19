//! Gemma 4 E4B (MASTER_PLAN VD4): it describes figures (D13, D26) and runs the LLM pass (D25,
//! `passes`). Port of V0 `sift/describe/gemma.py` and `sift/refine/llm.py`.

pub mod passes;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::answers::{AnswerStore, request_key};
use crate::config::Gemma as GemmaCfg;
use crate::imaging::Rgb;
use crate::llm::{Completion, LlamaServer, StateFn, chat, chat_payload, detect_repeat_token, image_message};
use crate::py;

pub struct Variant {
    pub key: &'static str,
    pub name: &'static str,
    pub folder: PathBuf,
    pub model: &'static str,
    pub mmproj: &'static str,
    pub drafts: Vec<(&'static str, &'static str)>,
    pub default_draft: &'static str,
    /// Where the files come from: the user's choice, Unsloth's QAT GGUFs (Dynamic Q4_K_XL).
    pub repo: &'static str,
}

/// Gemma 4 E4B QAT (Unsloth's GGUF): the only Gemma Sift runs. Its folder is
/// `<models folder>\gemma-4-E4B-it-qat-GGUF` (empty when no models folder is set yet).
pub fn variant(_key: &str) -> Variant {
    Variant {
        key: "e4b",
        name: "Gemma 4 E4B (QAT Q4_K_XL)",
        folder: crate::paths::find_model_dir("gemma-4-E4B-it-qat-GGUF").unwrap_or_default(),
        model: "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf",
        mmproj: "mmproj-F16.gguf",
        drafts: vec![("mtp", "mtp-gemma-4-E4B-it.gguf")],
        default_draft: "mtp",
        repo: "unsloth/gemma-4-E4B-it-qat-GGUF",
    }
}

impl Variant {
    pub fn draft_file(&self, cfg: &GemmaCfg) -> &'static str {
        let role =
            cfg.draft.as_deref().filter(|d| self.drafts.iter().any(|(r, _)| r == d)).unwrap_or(self.default_draft);
        self.drafts.iter().find(|(r, _)| *r == role).map(|(_, f)| *f).unwrap_or(self.drafts[0].1)
    }
    pub fn missing(&self, cfg: &GemmaCfg) -> Vec<String> {
        let mut files = vec![self.model, self.mmproj];
        if cfg.mtp {
            files.push(self.draft_file(cfg));
        }
        files.into_iter().filter(|f| !self.folder.join(f).exists()).map(String::from).collect()
    }
}

/// llama-server flags (V0 gemma.server_args, from the user's benchmarked launcher).
pub fn server_args(cfg: &GemmaCfg) -> Vec<String> {
    let v = variant(&cfg.model);
    let p = cfg.parallel;
    let mut a: Vec<String> = [
        "-m",
        &v.folder.join(v.model).display().to_string(),
        "--mmproj",
        &v.folder.join(v.mmproj).display().to_string(),
        "-ngl",
        "-1",
        "--flash-attn",
        "on",
        "--jinja",
        "--reasoning",
        &cfg.reasoning,
        "--parallel",
        &p.to_string(),
        "-c",
        &(p * cfg.ctx_per_slot).to_string(),
        "--no-context-shift",
        "-b",
        &cfg.batch.to_string(),
        "-ub",
        &cfg.ubatch.to_string(),
        "-ctk",
        &cfg.kv_type,
        "-ctv",
        &cfg.kv_type,
        "--temp",
        &py::float_repr(cfg.temperature),
        "--top-p",
        &py::float_repr(cfg.top_p),
        "--top-k",
        &cfg.top_k.to_string(),
        "--min-p",
        &py::float_repr(cfg.min_p),
        "--alias",
        "gemma",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if cfg.mtp {
        a.extend(
            [
                "--model-draft",
                &v.folder.join(v.draft_file(cfg)).display().to_string(),
                "--spec-type",
                "draft-mtp",
                "--spec-draft-n-max",
                &cfg.spec_draft_n_max.to_string(),
                "-ngld",
                "-1",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    }
    if let Some(n) = cfg.image_min_tokens {
        a.extend(["--image-min-tokens".into(), n.to_string()]);
    }
    if let Some(n) = cfg.image_max_tokens {
        a.extend(["--image-max-tokens".into(), n.to_string()]);
    }
    if let Some(r) = cfg.repeat_penalty {
        a.extend(["--repeat-penalty".into(), py::float_repr(r)]);
    }
    a
}

pub fn make_server(cfg: &GemmaCfg, build: &Path, log_dir: &Path, on_state: StateFn) -> LlamaServer {
    LlamaServer::new("gemma", server_args(cfg), build, log_dir, on_state)
}

fn repeated_lines(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().map(py::strip).filter(|l| !l.is_empty()).collect();
    let mut seen = std::collections::HashSet::new();
    lines.len() - lines.iter().filter(|l| seen.insert(**l)).count()
}

pub fn is_looping(c: &Completion) -> bool {
    let t = py::rstrip(&c.text);
    !t.is_empty()
        && (detect_repeat_token(t, 0) || (c.finish_reason.as_deref() == Some("length") && repeated_lines(t) > 0))
}

pub fn trim_repetition(text: &str, max_repeats: usize) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<&str> = Vec::new();
    let mut run = 0;
    let all = py::splitlines(text);
    for line in all.iter().map(|s| s.as_str()) {
        let key = py::strip(line);
        if !key.is_empty() && seen.contains(key) {
            run += 1;
            if run > max_repeats {
                break;
            }
        } else {
            run = 0;
        }
        if !key.is_empty() {
            seen.insert(key);
        }
        out.push(line);
    }
    while let Some(last) = out.last() {
        let k = py::strip(last);
        if !k.is_empty() && seen.contains(k) && run > 0 {
            out.pop();
            run -= 1;
        } else {
            break;
        }
    }
    format!("{}\n(Description cut: the model started repeating itself.)", py::rstrip(&out.join("\n")))
}

fn cut_at_last_line(text: &str) -> String {
    let mut t = py::rstrip(text).to_string();
    if let Some(i) = t.rfind('\n') {
        t = py::rstrip(&t[..i]).to_string();
    }
    format!("{t}\n(Description cut at the length limit.)")
}

fn sampling(cfg: &GemmaCfg) -> Vec<(&'static str, Value)> {
    let mut s = vec![
        ("temperature", json!(cfg.temperature)),
        ("top_p", json!(cfg.top_p)),
        ("top_k", json!(cfg.top_k)),
        ("min_p", json!(cfg.min_p)),
    ];
    if let Some(r) = cfg.repeat_penalty {
        s.push(("repeat_penalty", json!(r)));
    }
    s
}

/// One description (V0 gemma.describe): errors retried; a loop or a cut answer retried once
/// warmer with a repeat penalty, else trimmed.
pub fn describe_live(server: &LlamaServer, png: &[u8], prompt: &str, cfg: &GemmaCfg) -> Completion {
    let msgs = image_message(png, prompt);
    let s = sampling(cfg);
    let mut c = chat(server, msgs.clone(), cfg.max_tokens, &s);
    let mut n = 0;
    while c.error.is_some() && n < 2 && !server.is_stopped() {
        n += 1;
        c = chat(server, msgs.clone(), cfg.max_tokens, &s);
    }
    if c.error.is_none() && (is_looping(&c) || c.finish_reason.as_deref() == Some("length")) {
        let mut retry = s.clone();
        for e in retry.iter_mut() {
            if e.0 == "temperature" {
                e.1 = json!(cfg.temperature.max(0.3));
            }
        }
        retry.retain(|e| e.0 != "repeat_penalty");
        retry.push(("repeat_penalty", json!(1.15)));
        let mut c2 = chat(server, msgs, cfg.max_tokens, &retry);
        c2.seconds += c.seconds;
        c2.retries = c.retries + 1;
        if c2.error.is_none() && c2.finish_reason.as_deref() == Some("stop") && !is_looping(&c2) {
            c = c2;
        }
        if is_looping(&c) {
            c.text = trim_repetition(&c.text, 3);
            c.finish_reason = Some("trimmed".into());
        } else if c.finish_reason.as_deref() == Some("length") {
            c.text = cut_at_last_line(&c.text);
            c.finish_reason = Some("trimmed".into());
        }
    }
    c.text = py::strip(&c.text).to_string();
    c
}

/// Description through the answer store.
pub fn describe(
    server: Option<&LlamaServer>,
    store: &AnswerStore,
    img: &Rgb,
    png: &[u8],
    prompt: &str,
    cfg: &GemmaCfg,
) -> Completion {
    let key = request_key("gemma-describe", prompt, &[img.pixel_hash()], cfg.max_tokens, "");
    if let Some(v) = store.get(&key) {
        return Completion {
            text: v["text"].as_str().unwrap_or("").to_string(),
            cached: true,
            finish_reason: Some("stop".into()),
            ..Default::default()
        };
    }
    let Some(server) = server.filter(|_| !store.replay_only) else {
        store.note_miss(format!(
            "gemma-describe {} {}",
            img.pixel_hash(),
            serde_json::to_string(prompt).unwrap_or_default()
        ));
        return Completion { error: Some("not in the answer store / no brain server".into()), ..Default::default() };
    };
    let c = describe_live(server, png, prompt, cfg);
    if c.error.is_none() && !c.text.is_empty() {
        store.put(&key, "gemma-describe", prompt, cfg.max_tokens, "", &json!({"text": c.text}));
    }
    c
}

/// Figure kind when caption and figure text don't settle it.
pub fn classify(server: Option<&LlamaServer>, store: &AnswerStore, img: &Rgb, png: &[u8]) -> Option<String> {
    let p = crate::describe::CLASSIFY_PROMPT;
    let key = request_key("gemma-classify", p, &[img.pixel_hash()], 8, "");
    if let Some(v) = store.get(&key) {
        return v["kind"].as_str().map(String::from);
    }
    let server = server.filter(|_| !store.replay_only)?;
    let c = chat(server, image_message(png, p), 8, &[("temperature", json!(0.0)), ("top_k", json!(1))]);
    let kind = crate::describe::classify_answer(&c.text).to_string();
    if c.error.is_none() {
        store.put(&key, "gemma-classify", p, 8, "", &json!({"kind": kind}));
    }
    Some(kind)
}

pub const THINK_TOKENS: u32 = 4096;

/// A JSON answer that follows `schema` (llama-server turns it into a grammar), or None.
#[allow(clippy::too_many_arguments)]
pub fn ask_json(
    server: Option<&LlamaServer>,
    store: &AnswerStore,
    prompt: &str,
    schema: &Value,
    images: &[(Rgb, Vec<u8>)],
    max_tokens: u32,
    task: &str,
    think: bool,
) -> (Option<Value>, Completion) {
    let extra = format!("{task}|{}", think as u8);
    let hashes: Vec<String> = images.iter().map(|(r, _)| r.pixel_hash()).collect();
    let key = request_key("gemma-json", prompt, &hashes, max_tokens, &extra);
    if let Some(v) = store.get(&key) {
        let a = v.get("answer").cloned().filter(|a| !a.is_null());
        return (a, Completion { cached: true, ..Default::default() });
    }
    let Some(server) = server.filter(|_| !store.replay_only) else {
        store.note_miss(format!("gemma-json {task}"));
        return (
            None,
            Completion { error: Some("not in the answer store / no brain server".into()), ..Default::default() },
        );
    };
    use base64::Engine;
    let mut parts: Vec<Value> = images
        .iter()
        .map(|(_, png)| json!({"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png))}}))
        .collect();
    parts.push(json!({"type": "text", "text": prompt}));
    let mut payload = json!({
        "model": "gemma",
        "messages": [{"role": "user", "content": parts}],
        "max_tokens": max_tokens + if think { THINK_TOKENS } else { 0 },
        "temperature": 0.0,
        "top_k": 1,
        "cache_prompt": false,
        "response_format": {"type": "json_schema", "json_schema": {"name": "answer", "schema": schema, "strict": true}},
    });
    if think {
        payload["chat_template_kwargs"] = json!({"enable_thinking": true});
    }
    let c = chat_payload(server, &payload);
    let ans = if c.error.is_none() && c.finish_reason.as_deref() != Some("length") {
        serde_json::from_str::<Value>(&c.text).ok().filter(|v| !v.is_null())
    } else {
        None
    };
    if c.error.is_none() {
        store.put(&key, "gemma-json", prompt, max_tokens, &extra, &json!({"answer": ans}));
    }
    (ans, c)
}
