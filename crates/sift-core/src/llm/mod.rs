//! Model servers (MASTER_PLAN 4.1; V0 `sift/llm/`): one supervised llama-server per
//! loaded model, started when its first job is queued (D23), stopped as soon as its last
//! answer is in (D2), inside a kill-on-close Job Object (ZER0's `job_object.rs`), never
//! attached to an old server. Requests are OpenAI-style chat completions.

pub mod jobobject;

use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};

#[derive(Clone, Debug, Default)]
pub struct Completion {
    pub text: String,
    pub finish_reason: Option<String>,
    pub tokens: u64,
    pub seconds: f64,
    pub error: Option<String>,
    pub timings: Value,
    pub retries: u32,
    pub cached: bool,
    pub reasoning_chars: usize,
}

pub type StateFn = Arc<dyn Fn(&str, &str, Option<f64>) + Send + Sync>;

pub struct LlamaServer {
    pub name: String,
    pub args: Vec<String>,
    pub build_dir: PathBuf,
    pub path_dirs: Vec<PathBuf>,
    pub log_dir: PathBuf,
    pub startup_timeout: Duration,
    inner: Mutex<ServerInner>,
    ready_cv: Condvar,
    on_state: StateFn,
    http: reqwest::blocking::Client,
}

#[derive(Default)]
struct ServerInner {
    child: Option<Child>,
    port: u16,
    started: Option<Instant>,
    ready: bool,
    ready_after: f64,
    stopped: bool,
    log_path: Option<PathBuf>,
}

fn free_port() -> Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

/// llama-server's path in a build folder.
pub fn server_exe(build_dir: &Path) -> PathBuf {
    build_dir.join(if cfg!(windows) { "llama-server.exe" } else { "llama-server" })
}

/// The command line a server starts with: the build's llama-server, the model's arguments, the
/// local address. A free function so the window can show it without a `LlamaServer`, whose HTTP
/// client must never be created or dropped on Tauri's async runtime (tokio panics there).
pub fn command_line(build_dir: &Path, args: &[String], port: u16) -> Vec<String> {
    let mut v = vec![server_exe(build_dir).display().to_string()];
    v.extend(args.iter().cloned());
    v.extend(["--host".into(), "127.0.0.1".into(), "--port".into(), port.to_string()]);
    v
}

impl LlamaServer {
    pub fn new(name: &str, args: Vec<String>, build_dir: &Path, log_dir: &Path, on_state: StateFn) -> LlamaServer {
        LlamaServer {
            name: name.into(),
            args,
            build_dir: build_dir.into(),
            path_dirs: crate::engine::runtime_path_dirs(build_dir),
            log_dir: log_dir.into(),
            startup_timeout: Duration::from_secs(600),
            inner: Mutex::new(ServerInner::default()),
            ready_cv: Condvar::new(),
            on_state,
            http: reqwest::blocking::Client::builder().timeout(Duration::from_secs(900)).build().expect("http client"),
        }
    }

    pub fn exe(&self) -> PathBuf {
        server_exe(&self.build_dir)
    }

    pub fn command_line(&self, port: u16) -> Vec<String> {
        command_line(&self.build_dir, &self.args, port)
    }

    /// Spawn without waiting for the model to load (once).
    pub fn start(&self) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        if g.child.is_some() || g.started.is_some() {
            return Ok(());
        }
        if g.stopped {
            bail!("{}: server was stopped", self.name);
        }
        if !self.exe().exists() {
            bail!("llama-server not found at {}. Install the engine: sift engine install", self.exe().display());
        }
        let port = free_port()?;
        std::fs::create_dir_all(&self.log_dir)?;
        let stamp = crate::util::stamp();
        let log_path = self.log_dir.join(format!("{}-{stamp}-{port}.log", self.name));
        let mut log = std::fs::File::create(&log_path)?;
        let cl = self.command_line(port);
        writeln!(log, "{}\n", cl.join(" "))?;
        let mut cmd = Command::new(&cl[0]);
        cmd.args(&cl[1..]).current_dir(&self.build_dir).stdin(Stdio::null()).stdout(log.try_clone()?).stderr(log);
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut dirs: Vec<PathBuf> = self.path_dirs.clone();
        dirs.extend(std::env::split_paths(&path));
        cmd.env("PATH", std::env::join_paths(dirs)?);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
        }
        let child = cmd.spawn()?;
        jobobject::register(&child);
        g.child = Some(child);
        g.port = port;
        g.started = Some(Instant::now());
        g.log_path = Some(log_path);
        drop(g);
        (self.on_state)(&self.name, "starting", None);
        Ok(())
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.inner.lock().unwrap().port)
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        self.inner.lock().unwrap().log_path.clone()
    }

    pub fn ready_after(&self) -> f64 {
        self.inner.lock().unwrap().ready_after
    }

    fn log_tail(&self, n: usize) -> String {
        let Some(p) = self.log_path() else { return String::new() };
        let s = std::fs::read_to_string(p).unwrap_or_default();
        let lines: Vec<&str> = s.lines().collect();
        lines[lines.len().saturating_sub(n)..].join("\n")
    }

    pub fn wait_ready(&self, cancel: Option<&AtomicBool>) -> Result<()> {
        if self.inner.lock().unwrap().ready {
            return Ok(());
        }
        self.start()?;
        let deadline = Instant::now() + self.startup_timeout;
        while Instant::now() < deadline {
            if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                bail!("{}: cancelled while starting", self.name);
            }
            {
                let mut g = self.inner.lock().unwrap();
                if g.ready {
                    return Ok(());
                }
                let Some(child) = g.child.as_mut() else { bail!("{}: server was stopped while starting", self.name) };
                if let Ok(Some(status)) = child.try_wait() {
                    drop(g);
                    bail!("{} llama-server exited with {status}.\n{}", self.name, self.log_tail(25));
                }
            }
            let url = format!("{}/health", self.base_url());
            if let Ok(r) = self.http.get(&url).timeout(Duration::from_secs(2)).send()
                && r.status().is_success()
            {
                let first = {
                    let mut g = self.inner.lock().unwrap();
                    let first = !g.ready;
                    if first {
                        g.ready = true;
                        g.ready_after = g.started.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
                    }
                    first
                };
                self.ready_cv.notify_all();
                if first {
                    (self.on_state)(&self.name, "ready", Some(self.ready_after()));
                }
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        self.stop();
        bail!("{}: not ready after {:?}", self.name, self.startup_timeout)
    }

    /// Stop now. Callers stop a server only after its last answer (Cancel: after the queue is
    /// dropped), so nothing is waited for: a 50 ms look for an exit, then kill.
    pub fn stop(&self) {
        let child = {
            let mut g = self.inner.lock().unwrap();
            g.stopped = true;
            g.ready = false;
            g.child.take()
        };
        if let Some(mut c) = child {
            if let Ok(None) = c.try_wait() {
                let t0 = Instant::now();
                // llama-server has no graceful stop over HTTP; a short wait for in-flight
                // replies (there are none: callers stop after the last answer), then kill.
                while t0.elapsed() < Duration::from_millis(50) {
                    if let Ok(Some(_)) = c.try_wait() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                let _ = c.kill();
                let _ = c.wait();
            }
            (self.on_state)(&self.name, "stopped", None);
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.inner.lock().unwrap().stopped
    }

    pub fn chat(&self, payload: &Value) -> Result<Value> {
        let url = format!("{}/v1/chat/completions", self.base_url());
        let r = self.http.post(url).json(payload).send()?;
        let status = r.status();
        let body: Value = r.json().map_err(|e| anyhow!("bad reply ({status}): {e}"))?;
        if !status.is_success() {
            bail!("HTTP {status}: {body}");
        }
        Ok(body)
    }
}

impl Drop for LlamaServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// One chat request -> Completion (errors become `error`, never text).
pub fn chat(server: &LlamaServer, messages: Value, max_tokens: u32, sampling: &[(&str, Value)]) -> Completion {
    let mut payload = json!({"model": server.name, "messages": messages, "max_tokens": max_tokens});
    for (k, v) in sampling {
        if !v.is_null() {
            payload[*k] = v.clone();
        }
    }
    chat_payload(server, &payload)
}

pub fn chat_payload(server: &LlamaServer, payload: &Value) -> Completion {
    let t0 = Instant::now();
    match server.chat(payload) {
        Ok(r) => {
            let ch = &r["choices"][0];
            Completion {
                text: ch["message"]["content"].as_str().unwrap_or("").to_string(),
                finish_reason: ch["finish_reason"].as_str().map(String::from),
                tokens: r["usage"]["completion_tokens"].as_u64().unwrap_or(0),
                seconds: t0.elapsed().as_secs_f64(),
                timings: r["timings"].clone(),
                reasoning_chars: ch["message"]["reasoning_content"].as_str().map(|s| s.chars().count()).unwrap_or(0),
                ..Default::default()
            }
        }
        Err(e) => Completion { seconds: t0.elapsed().as_secs_f64(), error: Some(e.to_string()), ..Default::default() },
    }
}

pub fn image_message(png: &[u8], text: &str) -> Value {
    use base64::Engine;
    let url = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png));
    json!([{"role": "user", "content": [{"type": "image_url", "image_url": {"url": url}}, {"type": "text", "text": text}]}])
}

// ---------------------------------------------------------------------------------------
// ModelService: a worker pool per model, lazy start, stop when idle (V0 llm/services.py)
// ---------------------------------------------------------------------------------------

type Job = Box<dyn FnOnce(Option<&LlamaServer>) + Send>;

/// "Allowed to start" gate: Gemma starts after surya stopped (VD12).
#[derive(Default)]
pub struct Gate {
    open: Mutex<bool>,
    cv: Condvar,
}
impl Gate {
    pub fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.cv.notify_all();
    }
    pub fn wait(&self) {
        let mut g = self.open.lock().unwrap();
        while !*g {
            g = self.cv.wait(g).unwrap();
        }
    }
}

pub struct ModelService {
    pub name: String,
    pub server: Option<Arc<LlamaServer>>,
    gate: Option<Arc<Gate>>,
    on_idle_stop: Option<Arc<Gate>>,
    tx: Mutex<Option<Sender<Job>>>,
    pending: Arc<(Mutex<usize>, Condvar)>,
    finished: AtomicBool,
    stopped: Arc<AtomicBool>,
    pub requests: AtomicUsize,
    workers: Mutex<Vec<std::thread::JoinHandle<()>>>,
}

impl ModelService {
    pub fn new(
        name: &str,
        slots: usize,
        server: Option<Arc<LlamaServer>>,
        gate: Option<Arc<Gate>>,
        on_idle_stop: Option<Arc<Gate>>,
    ) -> Arc<ModelService> {
        let (tx, rx) = channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        let svc = Arc::new(ModelService {
            name: name.into(),
            server,
            gate,
            on_idle_stop,
            tx: Mutex::new(Some(tx)),
            pending: Arc::new((Mutex::new(0), Condvar::new())),
            finished: AtomicBool::new(false),
            stopped: Arc::new(AtomicBool::new(false)),
            requests: AtomicUsize::new(0),
            workers: Mutex::new(vec![]),
        });
        let mut ws = Vec::new();
        for i in 0..slots.max(1) {
            let rx: Arc<Mutex<Receiver<Job>>> = rx.clone();
            let s = Arc::downgrade(&svc);
            ws.push(
                std::thread::Builder::new()
                    .name(format!("{name}-{i}"))
                    .spawn(move || {
                        loop {
                            let job = { rx.lock().unwrap().recv() };
                            let Ok(job) = job else { break };
                            let Some(svc) = s.upgrade() else { break };
                            let ready = svc.ready();
                            match ready {
                                Ok(()) => job(svc.server.as_deref()),
                                Err(e) => {
                                    log::warn!("{}: {e}", svc.name);
                                    job(None)
                                }
                            }
                            svc.requests.fetch_add(1, Ordering::Relaxed);
                            svc.done_one();
                        }
                    })
                    .expect("spawn worker"),
            );
        }
        *svc.workers.lock().unwrap() = ws;
        svc
    }

    fn ready(&self) -> Result<()> {
        let Some(server) = &self.server else { return Ok(()) };
        if let Some(g) = &self.gate {
            g.wait();
        }
        if self.stopped.load(Ordering::Relaxed) {
            bail!("{}: stopped (cancelled): a model never starts after Cancel", self.name);
        }
        server.wait_ready(Some(&self.stopped))
    }

    /// Queue a job; it gets the ready server (None when the server failed or doesn't exist).
    pub fn submit(&self, job: impl FnOnce(Option<&LlamaServer>) + Send + 'static) -> Result<()> {
        if self.stopped.load(Ordering::Relaxed) {
            bail!("{}: already stopped", self.name);
        }
        *self.pending.0.lock().unwrap() += 1;
        let tx = self.tx.lock().unwrap();
        match tx.as_ref() {
            Some(t) => t.send(Box::new(job)).map_err(|_| anyhow!("{}: closed", self.name)),
            None => bail!("{}: closed", self.name),
        }
    }

    fn done_one(&self) {
        let idle = {
            let mut p = self.pending.0.lock().unwrap();
            *p -= 1;
            self.pending.1.notify_all();
            *p == 0 && self.finished.load(Ordering::Relaxed)
        };
        if idle {
            self.stop();
        }
    }

    /// No more work for this command: stop once the queue is empty.
    pub fn finish(&self) {
        self.finished.store(true, Ordering::Relaxed);
        if *self.pending.0.lock().unwrap() == 0 {
            self.stop();
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::Relaxed) {
            return;
        }
        if let Some(s) = &self.server {
            s.stop();
        }
        if let Some(g) = &self.on_idle_stop {
            g.open();
        }
    }

    /// Cancel: stop now; a worker waiting at the gate gives up (never starts a model).
    pub fn shutdown(&self) {
        self.stop();
        if let Some(g) = &self.gate {
            g.open();
        }
        self.tx.lock().unwrap().take();
    }
}

impl Drop for ModelService {
    fn drop(&mut self) {
        self.tx.lock().unwrap().take();
        if let Some(s) = &self.server {
            s.stop();
        }
    }
}

/// surya's detect_repeat_token (inference/util.py), on characters.
pub fn detect_repeat_token(text: &str, cut_from_end: usize) -> bool {
    let all: Vec<char> = text.chars().collect();
    let t: &[char] = if cut_from_end > 0 { &all[..all.len().saturating_sub(cut_from_end)] } else { &all };
    let window = 500usize;
    for seq_len in 1..=window / 2 {
        if t.len() < seq_len {
            continue;
        }
        let cand = &t[t.len() - seq_len..];
        let max_repeats = (4.0 * (1.0 + 3.0 / seq_len as f64)) as usize;
        let mut count = 0;
        let mut pos = t.len() as isize - seq_len as isize;
        while pos >= 0 {
            let p = pos as usize;
            if &t[p..p + seq_len] == cand {
                count += 1;
                pos -= seq_len as isize;
            } else {
                break;
            }
        }
        if count > max_repeats {
            return true;
        }
    }
    false
}
