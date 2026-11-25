//! The llama.cpp engine (MASTER_PLAN section 6): official builds only, installed or updated on
//! the user's request (`sift engine ...`, the Engine page) into the engine folder the user chose;
//! nothing is checked or downloaded on its own.
//!
//! Port of V0's `setup/engine.py` (itself the user's download-llama_v2.ps1, the same flow as
//! ZER0's llama_releases.rs): newest official release of ggml-org/llama.cpp for the detected
//! backend, streamed download, pure-Rust zip extraction into a temporary folder, verify
//! (`--version` reports the build, `--list-devices` shows the GPU) before switching
//! `current.txt`; the previous build is kept for `rollback`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;

use crate::py::Re;

pub const REPO: &str = "ggml-org/llama.cpp";
const UA: &str = "sift-engine-installer";

/// The user's engine folder (settings `[folders] engine_dir`; no default).
pub fn engine_dir() -> Result<PathBuf> {
    crate::paths::engine_dir()
}

fn current_file() -> Result<PathBuf> {
    Ok(engine_dir()?.join("current.txt"))
}
fn previous_file() -> Result<PathBuf> {
    Ok(engine_dir()?.join("previous.txt"))
}

pub fn current_build() -> Result<PathBuf> {
    let name = std::fs::read_to_string(current_file()?)
        .map_err(|_| anyhow!("No llama.cpp engine installed. Run: sift engine install"))?;
    let b = engine_dir()?.join(name.trim());
    if !b.exists() {
        bail!("Engine folder {} is missing. Run: sift engine install", b.display());
    }
    Ok(b)
}

/// Folders first on llama-server's PATH: the build, then the CUDA runtime if the build
/// doesn't carry its own (`cudart_dir.txt`, written at install).
pub fn runtime_path_dirs(build: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![build.to_path_buf()];
    let has_cudart = std::fs::read_dir(build)
        .map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().starts_with("cudart64_")))
        .unwrap_or(false);
    if !has_cudart && let Ok(d) = std::fs::read_to_string(build.join("cudart_dir.txt")) {
        let p = PathBuf::from(d.trim());
        if p.exists() {
            dirs.push(p);
        }
    }
    dirs
}

/// The CUDA version the NVIDIA driver supports (nvidia-smi), if any.
pub fn driver_cuda() -> Option<(u32, u32)> {
    crate::pyre_static!(CUDA, Re, r"CUDA (?:UMD )?Version:\s*(\d+)\.(\d+)");
    let o = Command::new("nvidia-smi").output().ok()?;
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    CUDA.search(&out).map(|m| (m[1].parse().unwrap_or(0), m[2].parse().unwrap_or(0)))
}

/// The backend family for this machine: `cuda` (the newest official CUDA build the driver
/// supports is chosen per release: llama.cpp moved from 13.4 to 13.5 in November 2026),
/// `vulkan` or `cpu`. An explicit `cuda-13.4` style backend pins one CUDA version.
pub fn detect_backend() -> String {
    if driver_cuda().is_some() {
        return "cuda".into();
    }
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    if Path::new(&sysroot).join("System32").join("vulkan-1.dll").exists() {
        return "vulkan".into();
    }
    "cpu".into()
}

fn http() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder().user_agent(UA).timeout(Duration::from_secs(1800)).build().expect("http")
}

fn get_json(url: &str) -> Result<Value> {
    let mut rq = http().get(url).header("Accept", "application/vnd.github+json");
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        rq = rq.bearer_auth(t);
    }
    let r = rq.send()?;
    if !r.status().is_success() {
        bail!("{url}: HTTP {}", r.status());
    }
    Ok(r.json()?)
}

fn build_number(rel: &Value) -> u64 {
    crate::pyre_static!(B, ReStart, r"b(\d+)");
    for k in ["tag_name", "name"] {
        if let Some(s) = rel[k].as_str()
            && let Some(m) = B.m(s)
        {
            return m[1].parse().unwrap_or(0);
        }
    }
    0
}

fn get_text(url: &str) -> Result<String> {
    let r = http().get(url).send()?;
    if !r.status().is_success() {
        bail!("{url}: HTTP {}", r.status());
    }
    Ok(r.text()?)
}

/// A release's assets read from its public release page (no API call): used when GitHub's
/// anonymous API limit (60 requests an hour) is used up.
fn release_from_page(tag: &str) -> Result<Value> {
    let html = get_text(&format!("https://github.com/{REPO}/releases/expanded_assets/{tag}"))?;
    let re = Re::new(&format!(
        r#"href="/{}/releases/download/{}/([^"/]+)""#,
        crate::py::re_escape(REPO),
        crate::py::re_escape(tag)
    ));
    let mut names: Vec<String> = re.find_all_group(&html, 1).into_iter().map(String::from).collect();
    names.sort();
    names.dedup();
    let assets: Vec<Value> = names
        .iter()
        .map(|n| serde_json::json!({"name": n, "browser_download_url": format!("https://github.com/{REPO}/releases/download/{tag}/{n}"), "size": 0}))
        .collect();
    Ok(serde_json::json!({"tag_name": tag, "assets": assets}))
}

/// Official build releases, newest first (the API, else the releases feed + release pages).
fn releases() -> Result<Vec<Value>> {
    match get_json(&format!("https://api.github.com/repos/{REPO}/releases?per_page=30")) {
        Ok(v) => {
            let mut a = v.as_array().cloned().unwrap_or_default();
            a.retain(|r| build_number(r) > 0);
            a.sort_by_key(|r| std::cmp::Reverse(build_number(r)));
            Ok(a)
        }
        Err(e) => {
            log::warn!("GitHub API: {e}; reading the releases feed instead");
            let feed = get_text(&format!("https://github.com/{REPO}/releases.atom"))?;
            crate::pyre_static!(TAG, Re, r"/releases/tag/(b\d+)");
            let mut tags: Vec<String> = TAG.find_all_group(&feed, 1).into_iter().map(String::from).collect();
            tags.sort_by_key(|t| std::cmp::Reverse(t[1..].parse::<u64>().unwrap_or(0)));
            tags.dedup();
            Ok(tags.into_iter().take(8).map(|t| serde_json::json!({"tag_name": t, "assets": null})).collect())
        }
    }
}

/// The newest official build release (or `tag`) that has a Windows asset for `backend`;
/// returns the release and the concrete backend (`cuda` -> `cuda-13.5`).
pub fn resolve_release(tag: Option<&str>, backend: &str) -> Result<(Value, String)> {
    let cands: Vec<Value> = match tag {
        Some(t) => vec![
            get_json(&format!("https://api.github.com/repos/{REPO}/releases/tags/{}", t.trim()))
                .or_else(|_| release_from_page(t.trim()))?,
        ],
        None => releases()?,
    };
    for mut rel in cands {
        if rel["assets"].is_null() {
            let t = rel["tag_name"].as_str().unwrap_or("").to_string();
            rel = release_from_page(&t)?;
        }
        if let Some(b) = concrete_backend(&rel, backend) {
            return Ok((rel, b));
        }
    }
    bail!("no recent official llama.cpp release has a Windows build for '{backend}'")
}

/// `cuda` -> the highest `cuda-X.Y` Windows x64 asset of this release that the driver supports.
fn concrete_backend(rel: &Value, backend: &str) -> Option<String> {
    let p = platform_tag();
    let names: Vec<String> =
        rel["assets"].as_array().into_iter().flatten().filter_map(|a| a["name"].as_str().map(String::from)).collect();
    if backend != "cuda" {
        let want = Re::new(&format!(r"^llama-.*-bin-{p}-{}-x64\.zip$", crate::py::re_escape(backend)));
        return names.iter().any(|n| want.is_match(n)).then(|| backend.to_string());
    }
    let re = Re::new(&format!(r"^llama-.*-bin-{p}-cuda-(\d+)\.(\d+)-x64\.zip$"));
    let drv = driver_cuda().unwrap_or((99, 99));
    names
        .iter()
        .filter_map(|n| re.search(n).map(|m| (m[1].parse::<u32>().unwrap_or(0), m[2].parse::<u32>().unwrap_or(0))))
        .filter(|v| *v <= drv)
        .max()
        .map(|(a, b)| format!("cuda-{a}.{b}"))
}

pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

fn asset(rel: &Value, pattern: &str, fallback: String) -> Asset {
    let re = Re::new(pattern);
    for a in rel["assets"].as_array().into_iter().flatten() {
        let n = a["name"].as_str().unwrap_or("");
        if re.is_match(n) {
            return Asset {
                name: n.into(),
                url: a["browser_download_url"].as_str().unwrap_or("").into(),
                size: a["size"].as_u64().unwrap_or(0),
            };
        }
    }
    let tag = rel["tag_name"].as_str().unwrap_or("");
    Asset { url: format!("https://github.com/{REPO}/releases/download/{tag}/{fallback}"), name: fallback, size: 0 }
}

pub fn platform_tag() -> &'static str {
    if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "ubuntu"
    }
}

pub fn main_asset(rel: &Value, backend: &str) -> Asset {
    let tag = rel["tag_name"].as_str().unwrap_or("");
    let esc = crate::py::re_escape(backend);
    let p = platform_tag();
    asset(rel, &format!(r"^llama-.*-bin-{p}-{esc}-x64\.zip$"), format!("llama-{tag}-bin-{p}-{backend}-x64.zip"))
}

pub fn cudart_asset(rel: &Value, backend: &str) -> Asset {
    let ver = backend.split_once('-').map(|x| x.1).unwrap_or("");
    asset(
        rel,
        &format!(r"^cudart-.*-cuda-{}-x64\.zip$", crate::py::re_escape(ver)),
        format!("cudart-llama-bin-win-cuda-{ver}-x64.zip"),
    )
}

/// A folder (CUDA_PATH, PATH) holding the CUDA runtime DLLs this backend needs.
pub fn system_cudart_dir(backend: &str) -> Option<PathBuf> {
    let ver = backend.strip_prefix("cuda-")?;
    let major = ver.split('.').next()?;
    let need = [format!("cudart64_{major}.dll"), format!("cublas64_{major}.dll")];
    let mut cands: Vec<PathBuf> = Vec::new();
    for var in ["CUDA_PATH".to_string(), format!("CUDA_PATH_V{}", ver.replace('.', "_"))] {
        if let Ok(b) = std::env::var(&var) {
            let b = PathBuf::from(b);
            cands.push(b.join("bin").join("x64"));
            cands.push(b.join("bin"));
        }
    }
    if let Some(p) = std::env::var_os("PATH") {
        cands.extend(std::env::split_paths(&p));
    }
    cands.into_iter().find(|c| need.iter().all(|d| c.join(d).exists()))
}

/// Streamed download with progress callback (bytes, total).
pub fn download(url: &str, dest: &Path, progress: &dyn Fn(u64, u64)) -> Result<()> {
    let mut r = http().get(url).send()?;
    if !r.status().is_success() {
        bail!("{url}: HTTP {}", r.status());
    }
    let total = r.content_length().unwrap_or(0);
    let tmp = dest.with_extension("part");
    let mut f = std::fs::File::create(&tmp)?;
    let mut buf = vec![0u8; 1 << 20];
    let mut done = 0u64;
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n])?;
        done += n as u64;
        progress(done, total);
    }
    drop(f);
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

fn extract_zip(zip_path: &Path, into: &Path) -> Result<()> {
    let f = std::fs::File::open(zip_path)?;
    let mut z = zip::ZipArchive::new(f)?;
    for i in 0..z.len() {
        let mut e = z.by_index(i)?;
        let Some(rel) = e.enclosed_name() else { continue };
        let out = into.join(rel);
        if e.is_dir() {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(p) = out.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut o = std::fs::File::create(&out)?;
        std::io::copy(&mut e, &mut o)?;
    }
    Ok(())
}

fn run_server(build: &Path, arg: &str) -> Result<String> {
    let exe = build.join(if cfg!(windows) { "llama-server.exe" } else { "llama-server" });
    let mut cmd = Command::new(&exe);
    cmd.arg(arg).current_dir(build);
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs = runtime_path_dirs(build);
    dirs.extend(std::env::split_paths(&path));
    cmd.env("PATH", std::env::join_paths(dirs)?);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let o = cmd.output().with_context(|| format!("running {}", exe.display()))?;
    Ok(format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)).trim().to_string())
}

pub fn server_version(build: &Path) -> Result<String> {
    run_server(build, "--version")
}

/// Verify before switching: version reports the tag; a CUDA build sees a CUDA device.
pub fn verify(build: &Path, tag: &str, backend: &str, log: &dyn Fn(&str)) -> Result<()> {
    crate::pyre_static!(BUILD, Re, r"build\s+(\d+)");
    crate::pyre_static!(OLD, Re, r"version:\s*(\d+)\s*\(");
    crate::pyre_static!(TAGN, ReStart, r"b(\d+)");
    let ver = server_version(build)?;
    let got = BUILD.search(&ver).or_else(|| OLD.search(&ver)).map(|m| m[1].to_string());
    let want = TAGN.m(tag).map(|m| m[1].to_string());
    if got.is_none() || (want.is_some() && got != want) {
        bail!("llama-server --version does not report {tag}:\n{ver}");
    }
    log(&format!("  version ok: {}", ver.lines().next().unwrap_or("?")));
    if backend.starts_with("cuda") {
        let dev = run_server(build, "--list-devices")?;
        if !dev.contains("CUDA") {
            bail!("llama-server --list-devices shows no CUDA device:\n{dev}");
        }
        log(&format!("  device ok: {}", dev.lines().find(|l| l.contains("CUDA0")).unwrap_or("").trim()));
    }
    Ok(())
}

pub struct Status {
    pub active: Option<String>,
    pub previous: Option<String>,
    pub version: Option<String>,
    /// None: no engine folder chosen yet
    pub dir: Option<PathBuf>,
}

pub fn status() -> Status {
    let read = |p: Result<PathBuf>| {
        p.ok().and_then(|p| std::fs::read_to_string(p).ok()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    };
    let active = read(current_file());
    let version =
        current_build().ok().and_then(|b| server_version(&b).ok()).and_then(|v| v.lines().next().map(String::from));
    Status { active, previous: read(previous_file()), version, dir: engine_dir().ok() }
}

/// The button: install (first run) or update to the newest official build (or `tag`).
pub fn install(
    tag: Option<&str>,
    backend: Option<&str>,
    with_cudart: bool,
    log: &dyn Fn(&str),
    progress: &dyn Fn(u64, u64),
) -> Result<PathBuf> {
    let family = backend.map(String::from).unwrap_or_else(detect_backend);
    let (rel, backend) = resolve_release(tag, &family)?;
    log(&format!("Backend: {backend}"));
    let build_tag = rel["tag_name"].as_str().unwrap_or("").to_string();
    log(&format!("Release: {build_tag}"));
    let name = format!("{build_tag}-{backend}");
    let dir = engine_dir()?;
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(&name);
    let active = std::fs::read_to_string(current_file()?).ok().map(|s| s.trim().to_string());
    let exe = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    if target.join(exe).exists() {
        log(&format!("{name} is already installed"));
    } else {
        let tmp = dir.join(format!(".tmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("build"))?;
        let res = (|| -> Result<()> {
            let a = main_asset(&rel, &backend);
            log(&format!("Downloading {} ({:.0} MB)", a.name, a.size as f64 / 1e6));
            let zp = tmp.join(&a.name);
            download(&a.url, &zp, progress)?;
            extract_zip(&zp, &tmp.join("build"))?;
            std::fs::remove_file(&zp)?;
            if backend.starts_with("cuda") {
                match (with_cudart, system_cudart_dir(&backend)) {
                    (false, Some(sys)) => {
                        log(&format!("CUDA runtime found at {}; no extra download", sys.display()));
                        std::fs::write(tmp.join("build").join("cudart_dir.txt"), sys.display().to_string())?;
                    }
                    _ => {
                        let c = cudart_asset(&rel, &backend);
                        log(&format!("Downloading {}", c.name));
                        let cz = tmp.join(&c.name);
                        download(&c.url, &cz, progress)?;
                        extract_zip(&cz, &tmp.join("build"))?;
                        std::fs::remove_file(&cz)?;
                    }
                }
            }
            let mut root = tmp.join("build");
            if !root.join(exe).exists() {
                root = find_file(&root, exe)
                    .and_then(|p| p.parent().map(Path::to_path_buf))
                    .ok_or_else(|| anyhow!("the archive has no {exe}"))?;
            }
            if target.exists() {
                std::fs::remove_dir_all(&target)?;
            }
            std::fs::rename(&root, &target)?;
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&tmp);
        res?;
    }
    log("Verifying before switching:");
    if let Err(e) = verify(&target, &build_tag, &backend, log) {
        if let Some(a) = &active
            && *a != name
        {
            log(&format!("Verification failed; {a} stays active."));
        }
        return Err(e);
    }
    if active.as_deref() != Some(name.as_str()) {
        if let Some(a) = &active {
            std::fs::write(previous_file()?, format!("{a}\n"))?;
        }
        std::fs::write(current_file()?, format!("{name}\n"))?;
    }
    // older builds stay until the user deletes them (`sift engine remove`, the Engine page)
    log(&format!("Active engine: {name}"));
    Ok(target)
}

fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    for e in std::fs::read_dir(root).ok()?.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = find_file(&p, name) {
                return Some(f);
            }
        } else if p.file_name().is_some_and(|f| f == name) {
            return Some(p);
        }
    }
    None
}

/// The backend to install: the settings' choice, `auto` = detected on this machine.
pub fn wanted_backend(setting: &str) -> String {
    if setting.trim().is_empty() || setting == "auto" { detect_backend() } else { setting.to_string() }
}

/// What a check found (no download).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Check {
    pub active: Option<String>,
    /// newest official build with a Windows asset for the chosen backend
    pub newest: String,
    pub backend: String,
    pub size: u64,
    pub published: String,
    pub update_available: bool,
    /// official builds now offer a newer CUDA version than the chosen one (e.g. cuda-13.6)
    pub newer_cuda: Option<String>,
}

/// Compare the newest official build for `backend` with the active one (VD9: reads the release
/// list only; the caller decides whether to update).
pub fn check(backend: &str) -> Result<Check> {
    let backend = wanted_backend(backend);
    let (rel, concrete) = resolve_release(None, &backend)?;
    let newest = format!("{}-{concrete}", rel["tag_name"].as_str().unwrap_or(""));
    let active = current_file()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let n = |s: &str| {
        s.strip_prefix('b').and_then(|x| x.split('-').next()).and_then(|x| x.parse::<u64>().ok()).unwrap_or(0)
    };
    let update_available = active.as_deref().is_none_or(|a| n(&newest) > n(a));
    let newer_cuda =
        if concrete.starts_with("cuda-") { concrete_backend(&rel, "cuda").filter(|c| *c != concrete) } else { None };
    Ok(Check {
        active,
        size: main_asset(&rel, &concrete).size,
        published: rel["published_at"].as_str().unwrap_or("").chars().take(10).collect(),
        newest,
        backend: concrete,
        update_available,
        newer_cuda,
    })
}

/// One installed build in the engine folder.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Installed {
    pub name: String,
    pub dir: PathBuf,
    pub size_mb: u64,
    /// CUDA runtime DLLs carried by the build itself (removable when a toolkit provides them)
    pub cuda_runtime_mb: u64,
    pub active: bool,
    pub previous: bool,
}

fn dir_size(p: &Path, only: &dyn Fn(&str) -> bool) -> u64 {
    std::fs::read_dir(p)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        dir_size(&path, only)
                    } else if only(&e.file_name().to_string_lossy()) {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    } else {
                        0
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn is_cuda_runtime(name: &str) -> bool {
    ["cudart64_", "cublas64_", "cublasLt64_"].iter().any(|p| name.starts_with(p))
}

/// The builds in the engine folder, newest first.
pub fn list() -> Result<Vec<Installed>> {
    let dir = engine_dir()?;
    let read = |p: Result<PathBuf>| {
        p.ok().and_then(|p| std::fs::read_to_string(p).ok()).map(|s| s.trim().to_string()).unwrap_or_default()
    };
    let (active, previous) = (read(current_file()), read(previous_file()));
    let exe = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    let mut out: Vec<Installed> = std::fs::read_dir(&dir)
        .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.path().join(exe).exists())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            Installed {
                size_mb: dir_size(&e.path(), &|_| true) / 1_000_000,
                cuda_runtime_mb: dir_size(&e.path(), &is_cuda_runtime) / 1_000_000,
                active: name == active,
                previous: name == previous,
                dir: e.path(),
                name,
            }
        })
        .collect();
    let n = |s: &str| {
        s.strip_prefix('b').and_then(|x| x.split('-').next()).and_then(|x| x.parse::<u64>().ok()).unwrap_or(0)
    };
    out.sort_by_key(|i| std::cmp::Reverse(n(&i.name)));
    Ok(out)
}

/// Make an installed build active, after the same verify step as an install.
pub fn use_build(name: &str, log: &dyn Fn(&str)) -> Result<PathBuf> {
    let dir = engine_dir()?;
    // a build of the engine folder's own list, never a path: `dir.join(name)` would accept an
    // absolute path or `..`, and the verify step below runs that folder's llama-server
    let target = list()?
        .into_iter()
        .find(|i| i.name == name)
        .map(|i| i.dir)
        .ok_or_else(|| anyhow!("{name} is not installed in {}", dir.display()))?;
    let (tag, backend) = name.split_once('-').unwrap_or((name, ""));
    verify(&target, tag, backend, log)?;
    let cur = std::fs::read_to_string(current_file()?).ok().map(|s| s.trim().to_string());
    if cur.as_deref() != Some(name) {
        if let Some(c) = &cur {
            std::fs::write(previous_file()?, format!("{c}\n"))?;
        }
        std::fs::write(current_file()?, format!("{name}\n"))?;
    }
    log(&format!("Active engine: {name}"));
    Ok(target)
}

/// Delete an installed build; the active and the previous build are refused.
pub fn remove(name: &str) -> Result<()> {
    let b = list()?.into_iter().find(|i| i.name == name).ok_or_else(|| anyhow!("{name} is not installed"))?;
    if b.active || b.previous {
        bail!("{name} is the {} build; switch to another build first", if b.active { "active" } else { "previous" });
    }
    std::fs::remove_dir_all(&b.dir)?;
    Ok(())
}

/// Delete the CUDA runtime DLLs a build carries, when an installed toolkit provides them.
pub fn remove_cuda_runtime(name: &str) -> Result<u64> {
    let b = list()?.into_iter().find(|i| i.name == name).ok_or_else(|| anyhow!("{name} is not installed"))?;
    let backend = name.split_once('-').map(|x| x.1).unwrap_or("");
    let sys = system_cudart_dir(backend).ok_or_else(|| {
        anyhow!("no CUDA toolkit with the runtime for {backend} was found; the build needs its own copy")
    })?;
    let mut freed = 0;
    for e in std::fs::read_dir(&b.dir)?.filter_map(|e| e.ok()) {
        if is_cuda_runtime(&e.file_name().to_string_lossy()) {
            freed += e.metadata().map(|m| m.len()).unwrap_or(0);
            std::fs::remove_file(e.path())?;
        }
    }
    std::fs::write(b.dir.join("cudart_dir.txt"), sys.display().to_string())?;
    Ok(freed / 1_000_000)
}

pub fn rollback(log: &dyn Fn(&str)) -> Result<PathBuf> {
    let prev =
        std::fs::read_to_string(previous_file()?).map_err(|_| anyhow!("No previous engine build to roll back to"))?;
    let prev = prev.trim().to_string();
    let exe = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    if !engine_dir()?.join(&prev).join(exe).exists() {
        bail!("Previous build {prev} is missing");
    }
    let cur = std::fs::read_to_string(current_file()?).ok().map(|s| s.trim().to_string());
    std::fs::write(current_file()?, format!("{prev}\n"))?;
    if let Some(c) = &cur {
        std::fs::write(previous_file()?, format!("{c}\n"))?;
    }
    log(&format!("Active engine: {prev}"));
    Ok(engine_dir()?.join(prev))
}
