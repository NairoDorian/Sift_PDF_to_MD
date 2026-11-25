//! The three models (MASTER_PLAN section 7): surya 2.5, surya's layout model and Gemma 4 E4B.
//! Each is pinned in `catalog/<id>.toml` (repository, revision, every file with its size and
//! sha256) and lives in `<models folder>\<folder>\`. Downloads come from the model's own
//! Hugging Face repository at the pinned revision, resume after an interruption and are checked
//! against sha256; nothing is downloaded before the user chose the models folder.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    /// path inside the repository (subfolders kept on disk, e.g. `order/order_ar.pt`)
    pub file: String,
    pub size: u64,
    pub sha256: String,
    /// a variant downloaded only on request (e.g. surya's BF16 weights)
    #[serde(default)]
    pub quant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    /// what Sift uses it for
    pub role: String,
    #[serde(default)]
    pub maker: String,
    #[serde(default)]
    pub license: String,
    pub repo: String,
    pub revision: String,
    /// folder name inside the models folder
    pub folder: String,
    pub files: Vec<FileRef>,
    #[serde(default)]
    pub optional: Vec<FileRef>,
}

/// What `status` found on disk.
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub id: String,
    pub name: String,
    pub role: String,
    pub license: String,
    pub revision: String,
    pub folder: Option<String>,
    pub size: u64,
    /// files missing or of the wrong size
    pub missing: Vec<String>,
    pub installed: bool,
}

impl Entry {
    pub fn dir(&self) -> Result<PathBuf> {
        crate::paths::find_model_dir(&self.folder)
    }
    pub fn local(&self, file: &str) -> Result<PathBuf> {
        Ok(self.dir()?.join(file))
    }
    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
    /// The required files plus the optional ones asked for (`quant`).
    pub fn wanted(&self, quant: Option<&str>) -> Vec<&FileRef> {
        let mut v: Vec<&FileRef> = self.files.iter().collect();
        if let Some(q) = quant {
            v.extend(self.optional.iter().filter(|f| f.quant.as_deref().is_some_and(|x| x.eq_ignore_ascii_case(q))));
        }
        v
    }
    /// Required files missing or of the wrong size (sizes are cheap; sha256: `verify`).
    pub fn missing(&self) -> Vec<String> {
        let Ok(dir) = self.dir() else { return self.files.iter().map(|f| f.file.clone()).collect() };
        self.files
            .iter()
            .filter(|f| std::fs::metadata(dir.join(&f.file)).map(|m| m.len() != f.size).unwrap_or(true))
            .map(|f| f.file.clone())
            .collect()
    }
    pub fn status(&self) -> Status {
        let missing = self.missing();
        Status {
            id: self.id.clone(),
            name: self.name.clone(),
            role: self.role.clone(),
            license: self.license.clone(),
            revision: self.revision.clone(),
            folder: self.dir().ok().map(|d| d.display().to_string()),
            size: self.size(),
            installed: missing.is_empty(),
            missing,
        }
    }
}

pub fn all() -> Result<Vec<Entry>> {
    let dir = crate::paths::catalog_dir();
    let mut out = Vec::new();
    let mut entries: Vec<_> =
        std::fs::read_dir(&dir).with_context(|| format!("model catalog in {}", dir.display()))?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("toml") {
            continue;
        }
        let s = std::fs::read_to_string(&p)?;
        out.push(toml::from_str::<Entry>(&s).with_context(|| format!("catalog entry {}", p.display()))?);
    }
    Ok(out)
}

pub fn get(id: &str) -> Result<Entry> {
    all()?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| anyhow!("no model '{id}' (have: surya-2.5, surya-layout, gemma-4-e4b-qat)"))
}

// ---------------------------------------------------------------------------------------
// downloads: resumable, sha256-checked
// ---------------------------------------------------------------------------------------

fn http() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("sift-models")
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(None)
        .build()
        .expect("http client")
}

pub fn file_url(repo: &str, revision: &str, file: &str) -> String {
    format!("https://huggingface.co/{repo}/resolve/{revision}/{file}")
}

fn sha256_file(p: &Path, progress: &dyn Fn(&str, u64, u64)) -> Result<String> {
    let mut f = std::fs::File::open(p)?;
    let total = f.metadata()?.len();
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 4 << 20];
    let mut done = 0u64;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        done += n as u64;
        progress("verify", done, total);
    }
    Ok(crate::imaging::hex(&h.finalize()))
}

/// Download one file into `dest`, resuming a `.part` file, then check size and sha256.
pub fn download_file(
    repo: &str,
    revision: &str,
    file: &str,
    size: u64,
    sha256: &str,
    dest: &Path,
    progress: &dyn Fn(&str, u64, u64),
) -> Result<()> {
    if std::fs::metadata(dest).map(|m| m.len() == size).unwrap_or(false) {
        return Ok(());
    }
    if let Some(d) = dest.parent() {
        std::fs::create_dir_all(d)?;
    }
    let part = dest.with_extension(format!("{}.part", dest.extension().and_then(|e| e.to_str()).unwrap_or("")));
    let have = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if have == size {
        // complete already (a run stopped while checking it): straight to the size and sha256 check
        return finish_part(&part, dest, file, size, sha256, progress);
    }
    let url = file_url(repo, revision, file);
    let mut req = http().get(&url);
    if have > 0 && have < size {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let mut r = req.send().with_context(|| url.clone())?;
    let status = r.status();
    let mut f = if status.as_u16() == 206 {
        let mut f = std::fs::OpenOptions::new().append(true).open(&part)?;
        f.seek(SeekFrom::End(0))?;
        f
    } else if status.is_success() {
        std::fs::File::create(&part)?
    } else {
        bail!("{url}: HTTP {status}");
    };
    let mut done = if status.as_u16() == 206 { have } else { 0 };
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n])?;
        done += n as u64;
        progress("download", done, size);
    }
    f.flush()?;
    drop(f);
    finish_part(&part, dest, file, size, sha256, progress)
}

/// A downloaded `.part` becomes the model file only with the pinned size and sha256.
fn finish_part(
    part: &Path,
    dest: &Path,
    file: &str,
    size: u64,
    sha256: &str,
    progress: &dyn Fn(&str, u64, u64),
) -> Result<()> {
    let got = std::fs::metadata(part)?.len();
    if got != size {
        bail!("{file}: got {got} bytes, expected {size} (run the download again to resume)");
    }
    let h = sha256_file(part, progress)?;
    if !h.eq_ignore_ascii_case(sha256) {
        let _ = std::fs::remove_file(part);
        bail!("{file}: sha256 {h} does not match the pinned {sha256}; the file was removed");
    }
    std::fs::rename(part, dest)?;
    Ok(())
}

/// Download every required file (and the optional `quant`), then record the revision.
pub fn download(e: &Entry, quant: Option<&str>, progress: &dyn Fn(&str, &str, u64, u64)) -> Result<()> {
    for f in e.wanted(quant) {
        let dest = e.local(&f.file)?;
        let name = f.file.clone();
        download_file(&e.repo, &e.revision, &f.file, f.size, &f.sha256, &dest, &|phase, d, t| {
            progress(&name, phase, d, t)
        })?;
    }
    std::fs::write(e.dir()?.join("REVISION.txt"), &e.revision)?;
    Ok(())
}

/// Re-check the sha256 of every file present: (file, ok). Missing files are reported as not ok.
pub fn verify(e: &Entry, progress: &dyn Fn(&str, &str, u64, u64)) -> Result<Vec<(String, bool)>> {
    let mut out = Vec::new();
    for f in e.files.iter().chain(e.optional.iter()) {
        let p = e.local(&f.file)?;
        if !p.exists() {
            if f.quant.is_none() {
                out.push((f.file.clone(), false));
            }
            continue;
        }
        let name = f.file.clone();
        let h = sha256_file(&p, &|phase, d, t| progress(&name, phase, d, t))?;
        out.push((f.file.clone(), h.eq_ignore_ascii_case(&f.sha256)));
    }
    Ok(out)
}

/// Delete a model's folder (only the files Sift put there and the folder itself).
pub fn delete(e: &Entry) -> Result<()> {
    let dir = e.dir()?;
    for f in e.files.iter().chain(e.optional.iter()) {
        let _ = std::fs::remove_file(dir.join(&f.file));
    }
    let _ = std::fs::remove_file(dir.join("REVISION.txt"));
    let _ = std::fs::remove_dir(dir.join("order"));
    let _ = std::fs::remove_dir(&dir);
    Ok(())
}

/// The repository's newest revision on Hugging Face (shown only; adopting it is a development
/// change: golden test + scorecard, MASTER_PLAN VD10).
pub fn upstream_revision(e: &Entry) -> Result<String> {
    let url = format!("https://huggingface.co/api/models/{}", e.repo);
    let r = http().get(&url).send().with_context(|| url.clone())?;
    if !r.status().is_success() {
        bail!("{url}: HTTP {}", r.status());
    }
    let v: serde_json::Value = r.json()?;
    v["sha"].as_str().map(String::from).ok_or_else(|| anyhow!("{url}: no revision in the answer"))
}
