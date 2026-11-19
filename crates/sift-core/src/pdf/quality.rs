//! Broken text-layer checks (V0 `sift/pdf/quality.py`, marker's check_line_spans and
//! detect_bad_ocr with the same thresholds).

use crate::imaging::Rgb;
use crate::py::{self};

const SPACE_THRESHOLD: f64 = 0.7;
const NEWLINE_THRESHOLD: f64 = 0.6;
const ALNUM_THRESHOLD: f64 = 0.3;

crate::pyre_static!(LEADERS, Re, r"(?:[.·•…_\-]\s*){3,}");
crate::pyre_static!(SPACES, Re, r"\s+");
crate::pyre_static!(NEWLINES, Re, r"\n+");

pub fn alphanum_ratio(text: &str) -> f64 {
    let t = text.replace([' ', '\n'], "");
    if t.is_empty() {
        return 1.0;
    }
    let n = t.chars().filter(|&c| py::is_alnum(c)).count();
    n as f64 / py::char_len(&t) as f64
}

pub fn private_use_count(text: &str) -> usize {
    text.chars().filter(|&c| (0xE000..=0xF8FF).contains(&(c as u32)) || c as u32 >= 0xF0000).count()
}

pub fn detect_bad_text(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    let text = LEADERS.replace_all(text, " ");
    let spaces = SPACES.find_all(&text).len() as f64;
    let alpha = py::char_len(&SPACES.replace_all(&text, "")) as f64;
    if spaces / (alpha + spaces).max(1.0) > SPACE_THRESHOLD {
        return true;
    }
    let newlines = NEWLINES.find_all(&text).len() as f64;
    let non_nl = py::char_len(&NEWLINES.replace_all(&text, "")) as f64;
    if newlines / (newlines + non_nl).max(1.0) > NEWLINE_THRESHOLD {
        return true;
    }
    if alphanum_ratio(&text) < ALNUM_THRESHOLD {
        return true;
    }
    let invalid = text.chars().filter(|&c| c == '\u{FFFD}').count() as f64;
    invalid > 6.0f64.max(py::char_len(&text) as f64 * 0.03)
}

pub fn page_text_ok<S: AsRef<str>>(line_texts: &[S]) -> bool {
    let mut text = String::new();
    for t in line_texts {
        text.push(' ');
        text.push_str(t.as_ref());
        text.push('\n');
    }
    if !py::has_ink(&text) {
        return false;
    }
    !detect_bad_text(&text)
}

pub fn block_text_bad(text: &str) -> bool {
    let t = py::strip(text);
    if t.is_empty() {
        return true;
    }
    if t.contains('\u{FFFD}') {
        return true;
    }
    let n = py::char_len(t);
    if private_use_count(t) as f64 > 2f64.max(n as f64 * 0.05) {
        return true;
    }
    n >= 12 && alphanum_ratio(&LEADERS.replace_all(t, " ")) < ALNUM_THRESHOLD
}

/// PIL `convert("L")` luminance (ITU-R 601-2, Pillow's fixed-point form).
#[inline]
pub fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((r as u32 * 19595 + g as u32 * 38470 + b as u32 * 7471 + 0x8000) >> 16) as u8
}

pub fn has_ink(img: &Rgb, white: u8, min_fraction: f64) -> bool {
    let n = img.width * img.height;
    if n == 0 {
        return false;
    }
    let dark = img.data.chunks_exact(3).filter(|p| luma(p[0], p[1], p[2]) < white).count();
    dark as f64 / n as f64 > min_fraction
}
