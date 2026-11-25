//! rf-detr layout detection (surya's `fast_layout`, `datalab-to/surya_layout2`), in Rust on
//! the CPU.
//!
//! A port of V0's vendored copy (`sift/layout/rfdetr/`: Roboflow RF-DETR, LW-DETR, Apache-2.0,
//! see NOTICE), inference only, configuration "rf-detr-large" as surya ships it:
//! - DINOv2-small, windowed (2x2 windows, full attention in blocks 3, 6, 9), patch 16,
//!   448 px input, position table 44x44 resized to 28x28 (antialiased bicubic);
//! - projector P4: C2f (3 bottlenecks, channel LayerNorm, SiLU) on the 4 feature maps;
//! - two-stage query selection (300), 4 decoder layers with 16-head single-level deformable
//!   cross-attention (2 points), lite reference points, box reparametrisation;
//! - post-processing: sigmoid, top-100 of queries x classes, confidence > 0.4.
//!
//! Layout runs inside Sift (no second inference runtime: plain matrix code on candle, CPU or
//! GPU), with surya's own weights, so the boxes are the ones surya's layout gives.

use std::path::Path;

use anyhow::{Result, bail};
use candle_core::{Device, Tensor};

use super::nn::{LayerNorm, Linear, Mha, Mlp, Weights, attention};
use crate::imaging::Rgb;

const RES: usize = 448;
const PATCH: usize = 16;
const GRID: usize = RES / PATCH; // 28
const POS: usize = 44;
const HID: usize = 384;
const HEADS: usize = 6;
const LAYERS: usize = 12;
const WINDOWS: usize = 2;
const WIN: usize = GRID / WINDOWS; // 14
const OUT_LAYERS: [usize; 4] = [3, 6, 9, 12]; // hidden_states indexes (after layers 2, 5, 8, 11)
const FULL_ATTN: [usize; 3] = [3, 6, 9];
const D: usize = 256;
const SA_HEADS: usize = 8;
const CA_HEADS: usize = 16;
const POINTS: usize = 2;
const DEC_LAYERS: usize = 4;
const QUERIES: usize = 300;
const NUM_SELECT: usize = 100;
pub const CONFIDENCE_THRESHOLD: f32 = 0.4;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

// ---------------------------------------------------------------------------------------
// Antialiased interpolation (torch F.interpolate(..., antialias=True))
// ---------------------------------------------------------------------------------------

fn aa_weights(in_size: usize, out_size: usize, interp: f32, filter: fn(f32) -> f32) -> Vec<(usize, Vec<f32>)> {
    let scale = in_size as f32 / out_size as f32;
    let support = if scale >= 1.0 { interp * 0.5 * scale } else { interp * 0.5 };
    let invscale = if scale >= 1.0 { 1.0 / scale } else { 1.0 };
    (0..out_size)
        .map(|i| {
            let center = scale * (i as f32 + 0.5);
            let xmin = ((center - support + 0.5) as i64).max(0);
            let xmax = ((center + support + 0.5) as i64).min(in_size as i64);
            let n = (xmax - xmin).max(0) as usize;
            let mut w: Vec<f32> = (0..n).map(|j| filter((j as f32 + xmin as f32 - center + 0.5) * invscale)).collect();
            let total: f32 = w.iter().sum();
            if total != 0.0 {
                for v in w.iter_mut() {
                    *v /= total;
                }
            }
            (xmin as usize, w)
        })
        .collect()
}

fn bilinear_filter(x: f32) -> f32 {
    let x = x.abs();
    if x < 1.0 { 1.0 - x } else { 0.0 }
}

fn bicubic_aa_filter(x: f32) -> f32 {
    const A: f32 = -0.5;
    let x = x.abs();
    if x < 1.0 {
        ((A + 2.0) * x - (A + 3.0)) * x * x + 1.0
    } else if x < 2.0 {
        (((x - 5.0) * x + 8.0) * x - 4.0) * A
    } else {
        0.0
    }
}

/// [C][H][W] planes -> [C][oh][ow], separable (x then y), f32.
// eight plain values, the arguments of the resize it ports (a struct would only rename them)
#[allow(clippy::too_many_arguments)]
fn resize_planes(
    src: &[f32],
    c: usize,
    h: usize,
    w: usize,
    oh: usize,
    ow: usize,
    interp: f32,
    f: fn(f32) -> f32,
) -> Vec<f32> {
    let wx = aa_weights(w, ow, interp, f);
    let wy = aa_weights(h, oh, interp, f);
    let mut out = vec![0f32; c * oh * ow];
    let mut row = vec![0f32; ow];
    for ch in 0..c {
        let plane = &src[ch * h * w..(ch + 1) * h * w];
        // horizontal pass of every source row, then vertical
        let mut tmp = vec![0f32; h * ow];
        for y in 0..h {
            let r = &plane[y * w..(y + 1) * w];
            for (ox, (xmin, ws)) in wx.iter().enumerate() {
                let mut s = 0f32;
                for (j, wv) in ws.iter().enumerate() {
                    s += r[xmin + j] * wv;
                }
                row[ox] = s;
            }
            tmp[y * ow..(y + 1) * ow].copy_from_slice(&row);
        }
        for (oy, (ymin, ws)) in wy.iter().enumerate() {
            let o = &mut out[ch * oh * ow + oy * ow..ch * oh * ow + (oy + 1) * ow];
            for (j, wv) in ws.iter().enumerate() {
                let t = &tmp[(ymin + j) * ow..(ymin + j + 1) * ow];
                for x in 0..ow {
                    o[x] += t[x] * wv;
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------------------

struct VitLayer {
    norm1: LayerNorm,
    q: Linear,
    k: Linear,
    v: Linear,
    o: Linear,
    ls1: Tensor,
    norm2: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    ls2: Tensor,
}

struct ConvLn {
    w: Tensor, // [out, in, k, k]
    k: usize,
    ln: LayerNorm,
}

impl ConvLn {
    fn new(w: &Weights, p: &str) -> Result<ConvLn> {
        let cw = w.get(&format!("{p}.conv.weight"))?;
        let k = cw.dim(2)?;
        Ok(ConvLn { w: cw, k, ln: LayerNorm::new(w, &format!("{p}.bn"), 1e-6)? })
    }
    /// x: [HW, Cin] (GRID x GRID) -> [HW, Cout], conv (no bias) + channel LN + SiLU
    fn fwd(&self, x: &Tensor) -> Result<Tensor> {
        let (hw, cin) = x.dims2()?;
        let cout = self.w.dim(0)?;
        let y = if self.k == 1 {
            x.matmul(&self.w.reshape((cout, cin))?.t()?)?
        } else {
            let img = x.t()?.contiguous()?.reshape((1, cin, GRID, GRID))?;
            let o = img.conv2d(&self.w, self.k / 2, 1, 1, 1)?; // [1, cout, H, W]
            o.reshape((cout, hw))?.t()?.contiguous()?
        };
        let y = self.ln.fwd(&y)?;
        Ok(y.silu()?)
    }
}

struct DecLayer {
    sa: Mha,
    norm1: LayerNorm,
    off: Linear,
    attw: Linear,
    vproj: Linear,
    oproj: Linear,
    lin1: Linear,
    lin2: Linear,
    norm2: LayerNorm,
    norm3: LayerNorm,
}

pub struct RfDetr {
    patch_w: Tensor, // [768, 384]
    patch_b: Tensor,
    cls: Tensor, // [1, 384] cls token + its position
    pos: Tensor, // [784, 384] patch positions resized to 28x28
    layers: Vec<VitLayer>,
    final_ln: LayerNorm,
    p_cv1: ConvLn,
    p_m: Vec<(ConvLn, ConvLn)>,
    p_cv2: ConvLn,
    p_ln: LayerNorm,
    enc_output: Linear,
    enc_output_norm: LayerNorm,
    enc_class: Linear,
    enc_bbox: Mlp,
    refpoint: Tensor,   // [300, 4]
    query_feat: Tensor, // [300, 256]
    ref_point_head: Mlp,
    dec: Vec<DecLayer>,
    dec_norm: LayerNorm,
    class_embed: Linear,
    bbox_embed: Mlp,
    pub num_classes: usize,
    /// where the backbone runs (the decoder always runs on the CPU)
    pub dev: Device,
}

#[derive(Clone, Debug)]
pub struct Detection {
    pub label: usize,
    pub score: f32,
    /// x0 y0 x1 y1 in pixels of the image given to `detect`
    pub bbox: [f32; 4],
}

pub struct PageResult {
    pub dets: Vec<Detection>,
    /// projector output [784, 256] (for the reading-order head)
    pub features: Tensor,
}

impl RfDetr {
    pub fn load(path: &Path) -> Result<RfDetr> {
        let w = Weights::load_pth(path, Some("model"))?;
        let dev = layout_device();
        let wb = w.on(&dev)?;
        let e = "backbone.0.encoder.encoder";
        let pw = w.get(&format!("{e}.embeddings.patch_embeddings.projection.weight"))?; // [384,3,16,16]
        if pw.dims() != [HID, 3, PATCH, PATCH] {
            bail!("unexpected rf-detr patch embedding {:?}", pw.dims());
        }
        let patch_w = pw.reshape((HID, 3 * PATCH * PATCH))?.t()?.contiguous()?.to_device(&dev)?;
        let patch_b = wb.get(&format!("{e}.embeddings.patch_embeddings.projection.bias"))?;
        let pe = w.get(&format!("{e}.embeddings.position_embeddings"))?; // [1, 1+44*44, 384]
        let pe = pe.squeeze(0)?;
        let cls_pos = pe.narrow(0, 0, 1)?;
        // [1936, 384] -> planes [384][44][44] -> bicubic AA -> [784, 384]
        let grid = pe.narrow(0, 1, POS * POS)?.t()?.contiguous()?.flatten_all()?.to_vec1::<f32>()?;
        let r = resize_planes(&grid, HID, POS, POS, GRID, GRID, 4.0, bicubic_aa_filter);
        let pos = Tensor::from_vec(r, (HID, GRID * GRID), &Device::Cpu)?.t()?.contiguous()?.to_device(&dev)?;
        let cls = (w.get(&format!("{e}.embeddings.cls_token"))?.reshape((1, HID))? + cls_pos)?.to_device(&dev)?;
        let mut layers = Vec::new();
        for i in 0..LAYERS {
            let p = format!("{e}.encoder.layer.{i}");
            layers.push(VitLayer {
                norm1: LayerNorm::new(&wb, &format!("{p}.norm1"), 1e-6)?,
                q: Linear::new(&wb, &format!("{p}.attention.attention.query"))?,
                k: Linear::new(&wb, &format!("{p}.attention.attention.key"))?,
                v: Linear::new(&wb, &format!("{p}.attention.attention.value"))?,
                o: Linear::new(&wb, &format!("{p}.attention.output.dense"))?,
                ls1: wb.get(&format!("{p}.layer_scale1.lambda1"))?,
                norm2: LayerNorm::new(&wb, &format!("{p}.norm2"), 1e-6)?,
                fc1: Linear::new(&wb, &format!("{p}.mlp.fc1"))?,
                fc2: Linear::new(&wb, &format!("{p}.mlp.fc2"))?,
                ls2: wb.get(&format!("{p}.layer_scale2.lambda1"))?,
            });
        }
        let pr = "backbone.0.projector.stages.0";
        let p_m = (0..3)
            .map(|i| {
                Ok((ConvLn::new(&wb, &format!("{pr}.0.m.{i}.cv1"))?, ConvLn::new(&wb, &format!("{pr}.0.m.{i}.cv2"))?))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut dec = Vec::new();
        for i in 0..DEC_LAYERS {
            let p = format!("transformer.decoder.layers.{i}");
            dec.push(DecLayer {
                sa: Mha::new(&w, &format!("{p}.self_attn"), SA_HEADS)?,
                norm1: LayerNorm::new(&w, &format!("{p}.norm1"), 1e-5)?,
                off: Linear::new(&w, &format!("{p}.cross_attn.sampling_offsets"))?,
                attw: Linear::new(&w, &format!("{p}.cross_attn.attention_weights"))?,
                vproj: Linear::new(&w, &format!("{p}.cross_attn.value_proj"))?,
                oproj: Linear::new(&w, &format!("{p}.cross_attn.output_proj"))?,
                lin1: Linear::new(&w, &format!("{p}.linear1"))?,
                lin2: Linear::new(&w, &format!("{p}.linear2"))?,
                norm2: LayerNorm::new(&w, &format!("{p}.norm2"), 1e-5)?,
                norm3: LayerNorm::new(&w, &format!("{p}.norm3"), 1e-5)?,
            });
        }
        let class_embed = Linear::new(&w, "class_embed")?;
        let num_classes = w.get("class_embed.bias")?.dim(0)?;
        Ok(RfDetr {
            patch_w,
            patch_b,
            cls,
            pos,
            layers,
            final_ln: LayerNorm::new(&wb, &format!("{e}.layernorm"), 1e-6)?,
            p_cv1: ConvLn::new(&wb, &format!("{pr}.0.cv1"))?,
            p_m,
            p_cv2: ConvLn::new(&wb, &format!("{pr}.0.cv2"))?,
            p_ln: LayerNorm::new(&wb, &format!("{pr}.1"), 1e-6)?,
            enc_output: Linear::new(&w, "transformer.enc_output.0")?,
            enc_output_norm: LayerNorm::new(&w, "transformer.enc_output_norm.0", 1e-5)?,
            enc_class: Linear::new(&w, "transformer.enc_out_class_embed.0")?,
            enc_bbox: Mlp::new(&w, "transformer.enc_out_bbox_embed.0", 3)?,
            refpoint: w.get("refpoint_embed.weight")?.narrow(0, 0, QUERIES)?,
            query_feat: w.get("query_feat.weight")?.narrow(0, 0, QUERIES)?,
            ref_point_head: Mlp::new(&w, "transformer.decoder.ref_point_head", 2)?,
            dec,
            dec_norm: LayerNorm::new(&w, "transformer.decoder.norm", 1e-5)?,
            class_embed,
            bbox_embed: Mlp::new(&w, "bbox_embed", 3)?,
            num_classes,
            dev,
        })
    }

    /// predictor.py preprocessing: to_tensor (/255 on the CPU), ImageNet normalise, resize
    /// to 448x448 (bilinear, antialias). Returns [3][448][448].
    fn preprocess(img: &Rgb) -> Vec<f32> {
        let (w, h) = (img.width, img.height);
        let mut planes = vec![0f32; 3 * w * h];
        for (i, p) in img.data.as_chunks::<3>().0.iter().enumerate() {
            for c in 0..3 {
                planes[c * w * h + i] = (p[c] as f32 / 255.0 - MEAN[c]) / STD[c];
            }
        }
        resize_planes(&planes, 3, h, w, RES, RES, 2.0, bilinear_filter)
    }

    fn vit_layer(&self, l: &VitLayer, x: &Tensor, full: bool) -> Result<Tensor> {
        // x: [4 windows, 197, 384]
        let (nw, t, c) = x.dims3()?;
        let xn = l.norm1.fwd(x)?;
        let att = if full {
            let flat = xn.reshape((nw * t, c))?;
            let a = attention(&l.q.fwd(&flat)?, &l.k.fwd(&flat)?, &l.v.fwd(&flat)?, HEADS, None)?;
            l.o.fwd(&a)?.reshape((nw, t, c))?
        } else {
            let mut outs = Vec::with_capacity(nw);
            for wi in 0..nw {
                let s = xn.get(wi)?;
                let a = attention(&l.q.fwd(&s)?, &l.k.fwd(&s)?, &l.v.fwd(&s)?, HEADS, None)?;
                outs.push(l.o.fwd(&a)?);
            }
            Tensor::stack(&outs, 0)?
        };
        let h = (att.broadcast_mul(&l.ls1)? + x)?;
        let m = l.fc2.fwd(&l.fc1.fwd(&l.norm2.fwd(&h)?)?.gelu_erf()?)?;
        Ok((m.broadcast_mul(&l.ls2)? + h)?)
    }

    /// Windowed tokens [4, 197, C] -> spatial [784, C] (row-major 28x28), cls dropped.
    fn unwindow(x: &Tensor) -> Result<Tensor> {
        let c = x.dim(2)?;
        let t = x.narrow(1, 1, WIN * WIN)?; // [4, 196, C]
        let t = t.reshape((WINDOWS, WINDOWS, WIN, WIN, c))?; // [yw, xw, yi, xi, C]
        let t = t.permute((0, 2, 1, 3, 4))?.contiguous()?; // [yw, yi, xw, xi, C]
        Ok(t.reshape((GRID * GRID, c))?)
    }

    /// Backbone + projector: [784, 256] feature map (row-major 28x28).
    fn features(&self, img: &Rgb) -> Result<Tensor> {
        let tp = std::time::Instant::now();
        let px = Self::preprocess(img);
        let t_pre = tp.elapsed();
        let x = Tensor::from_vec(px, (3, GRID, PATCH, GRID, PATCH), &Device::Cpu)?.to_device(&self.dev)?;
        let patches = x.permute((1, 3, 0, 2, 4))?.contiguous()?.reshape((GRID * GRID, 3 * PATCH * PATCH))?;
        let tokens = (patches.matmul(&self.patch_w)?.broadcast_add(&self.patch_b)? + &self.pos)?; // [784, 384]
        // windows: [yw, yi, xw, xi] -> window (yw, xw) of 14x14 tokens, each with the cls token
        let t = tokens.reshape((WINDOWS, WIN, WINDOWS, WIN, HID))?.permute((0, 2, 1, 3, 4))?.contiguous()?;
        let t = t.reshape((WINDOWS * WINDOWS, WIN * WIN, HID))?;
        let cls = self.cls.unsqueeze(0)?.repeat((WINDOWS * WINDOWS, 1, 1))?;
        let mut h = Tensor::cat(&[cls, t], 1)?; // [4, 197, 384]
        let mut feats = Vec::with_capacity(4);
        for (i, l) in self.layers.iter().enumerate() {
            if OUT_LAYERS.contains(&i) {
                feats.push(Self::unwindow(&self.final_ln.fwd(&h)?)?);
            }
            h = self.vit_layer(l, &h, FULL_ATTN.contains(&i))?;
        }
        feats.push(Self::unwindow(&self.final_ln.fwd(&h)?)?);
        let t_vit = tp.elapsed();
        let x = Tensor::cat(&feats, 1)?; // [784, 1536]
        // C2f
        let y = self.p_cv1.fwd(&x)?; // [784, 256]
        let c = y.dim(1)? / 2;
        let mut ys = vec![y.narrow(1, 0, c)?.contiguous()?, y.narrow(1, c, c)?.contiguous()?];
        for (a, b) in &self.p_m {
            let last = ys.last().unwrap().clone();
            ys.push(b.fwd(&a.fwd(&last)?)?);
        }
        let y = self.p_cv2.fwd(&Tensor::cat(&ys, 1)?)?;
        if std::env::var_os("SIFT_PROFILE_LAYOUT").is_some() {
            eprintln!(
                "  preprocess {:.0} ms, vit {:.0} ms, projector {:.0} ms",
                t_pre.as_secs_f64() * 1e3,
                (t_vit - t_pre).as_secs_f64() * 1e3,
                (tp.elapsed() - t_vit).as_secs_f64() * 1e3
            );
        }
        Ok(self.p_ln.fwd(&y)?.to_device(&Device::Cpu)?)
    }

    /// Two-stage proposals + decoder -> detections above `threshold`, boxes in pixels of `img`.
    pub fn detect(&self, img: &Rgb, threshold: f32) -> Result<PageResult> {
        let t0 = std::time::Instant::now();
        let memory = self.features(img)?; // [784, 256]
        let t_feat = t0.elapsed();
        let n = GRID * GRID;
        // encoder output proposals (bbox_reparam: not unsigmoided), all valid on a full page
        let mut props = Vec::with_capacity(n * 4);
        for y in 0..GRID {
            for x in 0..GRID {
                props.extend_from_slice(&[(x as f32 + 0.5) / GRID as f32, (y as f32 + 0.5) / GRID as f32, 0.05, 0.05]);
            }
        }
        let om = self.enc_output_norm.fwd(&self.enc_output.fwd(&memory)?)?;
        let cls = self.enc_class.fwd(&om)?; // [784, C]
        let best = cls.max(1)?.to_vec1::<f32>()?;
        let delta = self.enc_bbox.fwd(&om)?.to_vec2::<f32>()?;
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| best[b].partial_cmp(&best[a]).unwrap_or(std::cmp::Ordering::Equal));
        order.truncate(QUERIES);
        let rp = self.refpoint.to_vec2::<f32>()?;
        let mut refs = Vec::with_capacity(QUERIES * 4);
        for (qi, &i) in order.iter().enumerate() {
            let p = &props[i * 4..i * 4 + 4];
            let d = &delta[i];
            let ts = [d[0] * p[2] + p[0], d[1] * p[3] + p[1], d[2].exp() * p[2], d[3].exp() * p[3]];
            let s = &rp[qi];
            refs.extend_from_slice(&[
                s[0] * ts[2] + ts[0],
                s[1] * ts[3] + ts[1],
                s[2].exp() * ts[2],
                s[3].exp() * ts[3],
            ]);
        }
        let refs_t = Tensor::from_vec(refs.clone(), (QUERIES, 4), &Device::Cpu)?;
        let query_pos = self.ref_point_head.fwd(&sine_embed(&refs_t)?)?; // [300, 256]
        let mut tgt = self.query_feat.clone();
        for l in &self.dec {
            let q = (&tgt + &query_pos)?;
            let sa = l.sa.fwd(&q, &q, &tgt, None)?;
            tgt = l.norm1.fwd(&(tgt + sa)?)?;
            let qc = (&tgt + &query_pos)?;
            let ca = self.deform_attn(l, &qc, &memory, &refs)?;
            tgt = l.norm2.fwd(&(tgt + ca)?)?;
            let ff = l.lin2.fwd(&l.lin1.fwd(&tgt)?.relu()?)?;
            tgt = l.norm3.fwd(&(tgt + ff)?)?;
        }
        let hs = self.dec_norm.fwd(&tgt)?;
        if std::env::var_os("SIFT_PROFILE_LAYOUT").is_some() {
            eprintln!(
                "rf-detr: features {:.0} ms, decoder {:.0} ms",
                t_feat.as_secs_f64() * 1e3,
                (t0.elapsed() - t_feat).as_secs_f64() * 1e3
            );
        }
        let logits = self.class_embed.fwd(&hs)?.to_vec2::<f32>()?;
        let bd = self.bbox_embed.fwd(&hs)?.to_vec2::<f32>()?;
        let nc = self.num_classes;
        let mut flat: Vec<(f32, usize)> = Vec::with_capacity(QUERIES * nc);
        for (qi, row) in logits.iter().enumerate() {
            for (c, &v) in row.iter().enumerate() {
                flat.push((1.0 / (1.0 + (-v).exp()), qi * nc + c));
            }
        }
        flat.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let (iw, ih) = (img.width as f32, img.height as f32);
        let mut dets = Vec::new();
        for &(score, idx) in flat.iter().take(NUM_SELECT) {
            if score <= threshold {
                continue;
            }
            let (qi, label) = (idx / nc, idx % nc);
            let r = &refs[qi * 4..qi * 4 + 4];
            let d = &bd[qi];
            let cx = d[0] * r[2] + r[0];
            let cy = d[1] * r[3] + r[1];
            let w = (d[2].exp() * r[2]).max(0.0);
            let h = (d[3].exp() * r[3]).max(0.0);
            dets.push(Detection {
                label,
                score,
                bbox: [(cx - 0.5 * w) * iw, (cy - 0.5 * h) * ih, (cx + 0.5 * w) * iw, (cy + 0.5 * h) * ih],
            });
        }
        Ok(PageResult { dets, features: memory })
    }

    /// MSDeformAttn with one level, 4-d reference boxes, ms_deform_attn_core_pytorch
    /// (grid_sample bilinear, zero padding, align_corners=False).
    fn deform_attn(&self, l: &DecLayer, query: &Tensor, memory: &Tensor, refs: &[f32]) -> Result<Tensor> {
        let value = l.vproj.fwd(memory)?.to_vec2::<f32>()?; // [784, 256]
        let off = l.off.fwd(query)?.to_vec2::<f32>()?; // [300, heads*points*2]
        let aw = l.attw.fwd(query)?.to_vec2::<f32>()?; // [300, heads*points]
        let hd = D / CA_HEADS;
        let (hh, ww) = (GRID as f32, GRID as f32);
        let mut out = vec![0f32; QUERIES * D];
        for q in 0..QUERIES {
            let r = &refs[q * 4..q * 4 + 4];
            for h in 0..CA_HEADS {
                let a = &aw[q][h * POINTS..(h + 1) * POINTS];
                let m = a.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let e: Vec<f32> = a.iter().map(|v| (v - m).exp()).collect();
                let se: f32 = e.iter().sum();
                for p in 0..POINTS {
                    let o = &off[q][(h * POINTS + p) * 2..(h * POINTS + p) * 2 + 2];
                    let lx = r[0] + o[0] / POINTS as f32 * r[2] * 0.5;
                    let ly = r[1] + o[1] / POINTS as f32 * r[3] * 0.5;
                    let gx = 2.0 * lx - 1.0;
                    let gy = 2.0 * ly - 1.0;
                    let ix = ((gx + 1.0) * ww - 1.0) / 2.0;
                    let iy = ((gy + 1.0) * hh - 1.0) / 2.0;
                    let x0 = ix.floor();
                    let y0 = iy.floor();
                    let (x1, y1) = (x0 + 1.0, y0 + 1.0);
                    let wgt = e[p] / se;
                    let corners = [
                        (x0, y0, (x1 - ix) * (y1 - iy)),
                        (x1, y0, (ix - x0) * (y1 - iy)),
                        (x0, y1, (x1 - ix) * (iy - y0)),
                        (x1, y1, (ix - x0) * (iy - y0)),
                    ];
                    let dst = &mut out[q * D + h * hd..q * D + (h + 1) * hd];
                    for (cx, cy, bw) in corners {
                        if cx < 0.0 || cy < 0.0 || cx >= ww || cy >= hh {
                            continue;
                        }
                        let src = &value[cy as usize * GRID + cx as usize][h * hd..(h + 1) * hd];
                        for k in 0..hd {
                            dst[k] += wgt * bw * src[k];
                        }
                    }
                }
            }
        }
        l.oproj.fwd(&Tensor::from_vec(out, (QUERIES, D), &Device::Cpu)?)
    }
}

/// gen_sineembed_for_position(pos [N, 4], dim=128): (y, x, w, h) embeddings, 512 wide.
fn sine_embed(pos: &Tensor) -> Result<Tensor> {
    let p = pos.to_vec2::<f32>()?;
    let dim = 128usize;
    let dim_t: Vec<f32> = (0..dim).map(|i| 10000f32.powf(2.0 * ((i / 2) as f32) / dim as f32)).collect();
    let scale = 2.0 * std::f32::consts::PI;
    let mut out = Vec::with_capacity(p.len() * 512);
    for r in &p {
        for &comp in &[1usize, 0, 2, 3] {
            let e = r[comp] * scale;
            for (i, d) in dim_t.iter().enumerate() {
                let v = e / d;
                out.push(if i % 2 == 0 { v.sin() } else { v.cos() });
            }
        }
    }
    Ok(Tensor::from_vec(out, (p.len(), 512), &Device::Cpu)?)
}

/// The backbone's device: `SIFT_LAYOUT_DEVICE=cpu|cuda`, else CUDA when this build has it and a
/// GPU answers, else the CPU.
pub fn layout_device() -> Device {
    let want = std::env::var("SIFT_LAYOUT_DEVICE").unwrap_or_default().to_lowercase();
    if want == "cpu" {
        return Device::Cpu;
    }
    if candle_core::utils::cuda_is_available()
        && let Ok(d) = Device::new_cuda(0)
    {
        return d;
    }
    Device::Cpu
}
