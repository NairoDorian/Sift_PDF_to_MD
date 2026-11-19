//! Progress events (docs/events.md, shared with V0): `--json` prints one JSON object per
//! line on stdout; otherwise short human lines on stderr (V0 `sift/events.py`).

use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;

use serde_json::{Map, Value, json};

pub struct Events {
    pub json: bool,
    t0: Instant,
    last_stage: Mutex<std::collections::HashMap<(String, String), i64>>,
    lock: Mutex<()>,
    /// extra listeners (the window): every event as a JSON value
    pub sinks: Mutex<Vec<Box<dyn Fn(&Value) + Send>>>,
}

impl Events {
    pub fn new(json: bool) -> Events {
        Events {
            json,
            t0: Instant::now(),
            last_stage: Default::default(),
            lock: Mutex::new(()),
            sinks: Mutex::new(vec![]),
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.t0.elapsed().as_secs_f64()
    }

    pub fn emit(&self, kind: &str, data: Value) {
        let mut m = Map::new();
        m.insert("type".into(), json!(kind));
        m.insert("t".into(), json!((self.elapsed() * 1000.0).round() / 1000.0));
        if let Value::Object(o) = data {
            m.extend(o);
        }
        let v = Value::Object(m);
        for s in self.sinks.lock().unwrap().iter() {
            s(&v);
        }
        let _g = self.lock.lock().unwrap();
        if self.json {
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "{v}");
            let _ = out.flush();
        } else if let Some(line) = self.human(&v) {
            let mut e = std::io::stderr().lock();
            let _ = writeln!(e, "{line}");
        }
    }

    fn human(&self, d: &Value) -> Option<String> {
        let s = |k: &str| d[k].as_str().unwrap_or("").to_string();
        let n = |k: &str| d[k].as_i64().unwrap_or(0);
        match d["type"].as_str().unwrap_or("") {
            "job_start" => Some(format!(
                "Sift: {} file(s) -> {}  (mode {}, describer {}{})",
                n("files"),
                s("output"),
                s("mode"),
                s("describer"),
                if d["engine"].as_str().is_some_and(|e| !e.is_empty()) {
                    format!(", engine {}", s("engine"))
                } else {
                    String::new()
                }
            )),
            "file_start" => Some(format!("[{}/{}] {} ({} pages)", n("index") + 1, n("total"), s("file"), n("pages"))),
            "stage" => {
                let key = (s("file"), s("name"));
                let (done, total) = (n("done"), n("total"));
                let mut ls = self.last_stage.lock().unwrap();
                let last = *ls.get(&key).unwrap_or(&-1);
                let step = if total > 0 { (total / 10).max(1) } else { 1 };
                if done == 0 || done == total || done - last >= step {
                    ls.insert(key, done);
                    Some(format!("    {:<12} {done}/{total}", s("name")))
                } else {
                    None
                }
            }
            "server" => Some(format!(
                "    server {}: {}{}",
                s("model"),
                s("state"),
                d["seconds"].as_f64().map(|x| format!(" in {x:.1} s")).unwrap_or_default()
            )),
            "warning" => Some(format!("    warning: {}", s("message"))),
            "file_done" => Some(format!(
                "    done in {:.1} s: {} ({} images{})",
                d["seconds"].as_f64().unwrap_or(0.0),
                s("md"),
                n("images"),
                d["report"].as_str().map(|r| format!(", report {r}")).unwrap_or_default()
            )),
            "file_error" => Some(format!("    FAILED: {}", s("error"))),
            "job_done" => Some(format!(
                "Finished {}/{} file(s) in {:.1} s{}",
                n("ok"),
                n("files"),
                d["seconds"].as_f64().unwrap_or(0.0),
                if n("failed") > 0 { format!(", {} failed", n("failed")) } else { String::new() }
            )),
            "log" => Some(format!("    {}", s("message"))),
            _ => None,
        }
    }
}
