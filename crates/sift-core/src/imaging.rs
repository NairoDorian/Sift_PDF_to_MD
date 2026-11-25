//! Images: RGB buffers, crops, resampling, PNG.
//!
//! `resize_lanczos` is Pillow's `Image.resize(..., LANCZOS)` for 8-bit RGB, ported from
//! libImaging/Resample.c (fixed-point coefficients, horizontal pass then vertical pass on
//! the rows the vertical pass needs). surya's crops go through it (scale_to_fit on the
//! 28-px grid), so V1 sends surya the very pixels V0 sent: same answers, same answer-store
//! keys (5.4).

use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct Rgb {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Rgb {
    pub fn new(width: usize, height: usize, fill: u8) -> Rgb {
        Rgb { width, height, data: vec![fill; width * height * 3] }
    }

    /// PIL `img.crop((x0, y0, x1, y1))` with in-bounds integer boxes.
    pub fn crop(&self, x0: usize, y0: usize, x1: usize, y1: usize) -> Rgb {
        let (x1, y1) = (x1.min(self.width), y1.min(self.height));
        let (w, h) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
        let mut out = Vec::with_capacity(w * h * 3);
        for y in y0..y0 + h {
            let s = (y * self.width + x0) * 3;
            out.extend_from_slice(&self.data[s..s + w * 3]);
        }
        Rgb { width: w, height: h, data: out }
    }

    /// Every pixel has the colour of the first one: nothing is drawn here, there is nothing
    /// to read (a model asked to read such an image answers with text of its own).
    pub fn is_one_colour(&self) -> bool {
        let (pixels, _) = self.data.as_chunks::<3>();
        match pixels.first() {
            Some(first) => pixels.iter().all(|p| p == first),
            None => true,
        }
    }

    /// sha256 of "WxH|" + RGB bytes: the image part of an answer-store key.
    pub fn pixel_hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(format!("{}x{}|", self.width, self.height).as_bytes());
        h.update(&self.data);
        hex(&h.finalize())
    }

    /// Lossless PNG. `fast`: zlib level 1 (model requests, V0 speed rule 7); else default.
    pub fn png(&self, fast: bool) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, self.width as u32, self.height as u32);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(if fast { png::Compression::Fastest } else { png::Compression::Balanced });
            if fast {
                enc.set_filter(png::Filter::Sub);
            }
            let mut w = enc.write_header().expect("png header");
            w.write_image_data(&self.data).expect("png data");
        }
        buf
    }
}

pub fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

pub fn sha256_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}

// ------------------------------------------------------------------------------------
// Pillow Resample.c, LANCZOS, 8 bits per channel
// ------------------------------------------------------------------------------------

const PRECISION_BITS: i32 = 32 - 8 - 2;

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    let x = x * std::f64::consts::PI;
    x.sin() / x
}

fn lanczos(x: f64) -> f64 {
    if (-3.0..3.0).contains(&x) { sinc(x) * sinc(x / 3.0) } else { 0.0 }
}

struct Coeffs {
    ksize: usize,
    bounds: Vec<(usize, usize)>, // (xmin, count)
    kk: Vec<i32>,
}

fn precompute(in_size: usize, in0: f64, in1: f64, out_size: usize) -> Coeffs {
    let support_base = 3.0;
    let scale = (in1 - in0) / out_size as f64;
    let filterscale = if scale < 1.0 { 1.0 } else { scale };
    let support = support_base * filterscale;
    let ksize = (support.ceil() as usize) * 2 + 1;
    let mut pre = vec![0f64; out_size * ksize];
    let mut bounds = Vec::with_capacity(out_size);
    for xx in 0..out_size {
        let center = in0 + (xx as f64 + 0.5) * scale;
        let mut ww = 0.0;
        let ss = 1.0 / filterscale;
        let mut xmin = (center - support + 0.5) as i64; // C (int) truncation
        if xmin < 0 {
            xmin = 0;
        }
        let mut xmax = (center + support + 0.5) as i64;
        if xmax > in_size as i64 {
            xmax = in_size as i64;
        }
        let count = (xmax - xmin).max(0) as usize;
        let k = &mut pre[xx * ksize..(xx + 1) * ksize];
        // same order and arithmetic as Pillow's loop (its sums depend on both)
        for (x, kx) in k.iter_mut().enumerate().take(count) {
            let w = lanczos((x as f64 + xmin as f64 - center + 0.5) * ss);
            *kx = w;
            ww += w;
        }
        for v in k.iter_mut().take(count) {
            if ww != 0.0 {
                *v /= ww;
            }
        }
        bounds.push((xmin as usize, count));
    }
    let kk = pre
        .iter()
        .map(|&v| {
            let s = v * (1i64 << PRECISION_BITS) as f64;
            if v < 0.0 { (-0.5 + s) as i32 } else { (0.5 + s) as i32 }
        })
        .collect();
    Coeffs { ksize, bounds, kk }
}

#[inline]
fn clip8(v: i32) -> u8 {
    (v >> PRECISION_BITS).clamp(0, 255) as u8
}

/// One horizontal pass: rows [offset, offset+rows) of `src` -> `w` columns.
fn pass_h(src: &Rgb, w: usize, offset: usize, rows: usize, c: &Coeffs) -> Rgb {
    let mut tmp = Rgb::new(w, rows, 0);
    for yy in 0..rows {
        let iy = yy + offset;
        let inrow = &src.data[iy * src.width * 3..(iy + 1) * src.width * 3];
        let outrow = &mut tmp.data[yy * w * 3..(yy + 1) * w * 3];
        for xx in 0..w {
            let (xmin, cnt) = c.bounds[xx];
            let k = &c.kk[xx * c.ksize..xx * c.ksize + cnt];
            let half = 1i32 << (PRECISION_BITS - 1);
            let (mut s0, mut s1, mut s2) = (half, half, half);
            for (x, &kv) in k.iter().enumerate() {
                let p = (x + xmin) * 3;
                s0 = s0.wrapping_add(inrow[p] as i32 * kv);
                s1 = s1.wrapping_add(inrow[p + 1] as i32 * kv);
                s2 = s2.wrapping_add(inrow[p + 2] as i32 * kv);
            }
            outrow[xx * 3] = clip8(s0);
            outrow[xx * 3 + 1] = clip8(s1);
            outrow[xx * 3 + 2] = clip8(s2);
        }
    }
    tmp
}

/// One vertical pass: `src` -> `h` rows (bounds index rows of `src`).
fn pass_v(src: &Rgb, h: usize, c: &Coeffs) -> Rgb {
    let sw = src.width;
    let mut out = Rgb::new(sw, h, 0);
    for yy in 0..h {
        let (ymin, cnt) = c.bounds[yy];
        let k = &c.kk[yy * c.ksize..yy * c.ksize + cnt];
        let outrow = &mut out.data[yy * sw * 3..(yy + 1) * sw * 3];
        for xx in 0..sw {
            let half = 1i32 << (PRECISION_BITS - 1);
            let (mut s0, mut s1, mut s2) = (half, half, half);
            for (y, &kv) in k.iter().enumerate() {
                let p = ((y + ymin) * sw + xx) * 3;
                s0 = s0.wrapping_add(src.data[p] as i32 * kv);
                s1 = s1.wrapping_add(src.data[p + 1] as i32 * kv);
                s2 = s2.wrapping_add(src.data[p + 2] as i32 * kv);
            }
            outrow[xx * 3] = clip8(s0);
            outrow[xx * 3 + 1] = clip8(s1);
            outrow[xx * 3 + 2] = clip8(s2);
        }
    }
    out
}

/// Pillow 13 `img.resize((w, h), Image.Resampling.LANCZOS)` for an RGB image (box = whole
/// image), ImagingResampleInner: horizontal pass first (on the rows the vertical pass
/// needs), unless the height shrinks more than twice as much as the width.
pub fn resize_lanczos(img: &Rgb, w: usize, h: usize) -> Rgb {
    if w == img.width && h == img.height {
        return img.clone();
    }
    let need_h = w != img.width;
    let need_v = h != img.height;
    let (in_w, in_h) = (img.width as i64, img.height as i64);
    let horizontal_first = !((in_h - h as i64) > 0 && (in_h - h as i64) > (in_w - w as i64) * 2);
    let ch = precompute(img.width, 0.0, img.width as f64, w);
    let mut cv = precompute(img.height, 0.0, img.height as f64, h);
    if horizontal_first {
        let mut cur = img.clone();
        if need_h {
            let ybox_first = cv.bounds[0].0;
            let ybox_last = cv.bounds[h - 1].0 + cv.bounds[h - 1].1;
            for b in cv.bounds.iter_mut() {
                b.0 -= ybox_first;
            }
            cur = pass_h(img, w, ybox_first, ybox_last - ybox_first, &ch);
        }
        if need_v {
            cur = pass_v(&cur, h, &cv);
        }
        cur
    } else {
        let mut cur = img.clone();
        if need_v {
            cur = pass_v(img, h, &cv);
        }
        if need_h {
            cur = pass_h(&cur, w, 0, h, &ch);
        }
        cur
    }
}

/// surya's scale_to_fit (inference/util.py): fit the pixel count between min and max on the
/// 28-px grid, LANCZOS.
pub fn scale_to_fit(img: &Rgb) -> Rgb {
    scale_to_fit_with(img, (3072, 2048), (1792, 28), 28)
}

pub fn scale_to_fit_with(img: &Rgb, max_size: (usize, usize), min_size: (usize, usize), grid: usize) -> Rgb {
    let (width, height) = (img.width, img.height);
    if width == 0 || height == 0 {
        return img.clone();
    }
    let ar = width as f64 / height as f64;
    let cur = (width * height) as f64;
    let max_px = (max_size.0 * max_size.1) as f64;
    let min_px = (min_size.0 * min_size.1) as f64;
    let mut scale = 1.0;
    if cur > max_px {
        scale = (max_px / cur).powf(0.5);
    } else if cur < min_px {
        scale = (min_px / cur).powf(0.5);
    }
    let g = grid as f64;
    let mut wb = (crate::py::round_half_even(width as f64 * scale / g) as i64).max(1);
    let mut hb = (crate::py::round_half_even(height as f64 * scale / g) as i64).max(1);
    while (wb * hb) as f64 * g * g > max_px {
        if wb == 1 && hb == 1 {
            break;
        }
        if wb == 1 {
            hb -= 1;
            continue;
        }
        if hb == 1 {
            wb -= 1;
            continue;
        }
        let ar_w = (((wb - 1) as f64 / hb as f64) - ar).abs();
        let ar_h = ((wb as f64 / (hb - 1) as f64) - ar).abs();
        if ar_w < ar_h { wb -= 1 } else { hb -= 1 }
    }
    let (nw, nh) = (wb as usize * grid, hb as usize * grid);
    if (nw, nh) == (width, height) {
        return img.clone();
    }
    resize_lanczos(img, nw, nh)
}

/// V0's PdfDocument.crop: a PDF-point box (+pad) from a page rendered at `dpi`.
pub fn crop_pt(page: &Rgb, bbox: [f64; 4], dpi: f64, pad: f64) -> Rgb {
    let s = dpi / 72.0;
    let (w, h) = (page.width as i64, page.height as i64);
    let x0 = (((bbox[0] - pad) * s) as i64).max(0); // Python int() truncates toward zero
    let y0 = (((bbox[1] - pad) * s) as i64).max(0);
    let x1 = (crate::py::round_half_even((bbox[2] + pad) * s) as i64).min(w);
    let y1 = (crate::py::round_half_even((bbox[3] + pad) * s) as i64).min(h);
    if x1 <= x0 || y1 <= y0 {
        return page.crop(0, 0, 1, 1);
    }
    page.crop(x0 as usize, y0 as usize, x1 as usize, y1 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_sizes() {
        let img = Rgb::new(100, 50, 255);
        let s = scale_to_fit(&img);
        assert_eq!(s.width % 28, 0);
        assert_eq!(s.height % 28, 0);
        // a white image stays white through the filter
        assert!(s.data.iter().all(|&v| v == 255));
    }
}
