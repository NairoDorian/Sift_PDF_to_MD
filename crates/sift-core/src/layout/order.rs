//! surya's autoregressive reading-order head (`surya_layout2/order/order_ar.pt`), port of
//! V0's `sift/layout/order/` (Apache-2.0): box tokens (geometry + label) cross-attend to the
//! rf-detr projector map, then a greedy constrained decode emits the reading order. Used only
//! for pages without a text layer (marker and V0 order text pages by the PDF's own stream).

use std::path::Path;

use anyhow::Result;
use candle_core::{Device, Tensor};

use super::nn::{LayerNorm, Linear, Mha, Weights};

pub const MAX_BOXES: usize = 128;

/// The 19 raw rf-detr label names, sorted (order_ar.LAYOUT_CLASSES).
pub fn layout_classes() -> Vec<&'static str> {
    let mut v = vec![
        "Caption",
        "Footnote",
        "Equation-Block",
        "List-Group",
        "Page-Header",
        "Page-Footer",
        "Image",
        "Section-Header",
        "Table",
        "Text",
        "Complex-Block",
        "Code-Block",
        "Form",
        "Table-Of-Contents",
        "Figure",
        "Chemical-Block",
        "Diagram",
        "Bibliography",
        "Blank-Page",
    ];
    v.sort();
    v
}

struct DecoderLayer {
    sa: Mha,
    ca: Mha,
    l1: Linear,
    l2: Linear,
    n1: LayerNorm,
    n2: LayerNorm,
    n3: LayerNorm,
}

impl DecoderLayer {
    fn new(w: &Weights, p: &str) -> Result<DecoderLayer> {
        Ok(DecoderLayer {
            sa: Mha::new(w, &format!("{p}.self_attn"), 4)?,
            ca: Mha::new(w, &format!("{p}.multihead_attn"), 4)?,
            l1: Linear::new(w, &format!("{p}.linear1"))?,
            l2: Linear::new(w, &format!("{p}.linear2"))?,
            n1: LayerNorm::new(w, &format!("{p}.norm1"), 1e-5)?,
            n2: LayerNorm::new(w, &format!("{p}.norm2"), 1e-5)?,
            n3: LayerNorm::new(w, &format!("{p}.norm3"), 1e-5)?,
        })
    }
    /// torch.nn.TransformerDecoderLayer (post-norm, relu), batch of one.
    fn fwd(&self, x: &Tensor, mem: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let x = self.n1.fwd(&(x + self.sa.fwd(x, x, x, mask)?)?)?;
        let x = self.n2.fwd(&(&x + self.ca.fwd(&x, mem, mem, None)?)?)?;
        let ff = self.l2.fwd(&self.l1.fwd(&x)?.relu()?)?;
        self.n3.fwd(&(x + ff)?)
    }
}

pub struct OrderModel {
    geom: Linear,
    lab: Tensor,
    feat_proj: Linear,
    feat_pos: Tensor,
    ctx: Vec<DecoderLayer>,
    dec: Vec<DecoderLayer>,
    bos: Tensor,
    step: Tensor,
    out: Linear,
}

impl OrderModel {
    pub fn load(path: &Path) -> Result<OrderModel> {
        let w = Weights::load_pth(path, Some("model"))?;
        let n_layers = (0..8).take_while(|i| w.map.contains_key(&format!("ctx.layers.{i}.norm1.weight"))).count();
        Ok(OrderModel {
            geom: Linear::new(&w, "geom")?,
            lab: w.get("lab.weight")?,
            feat_proj: Linear::new(&w, "feat_proj")?,
            feat_pos: w.get("feat_pos")?.squeeze(0)?,
            ctx: (0..n_layers).map(|i| DecoderLayer::new(&w, &format!("ctx.layers.{i}"))).collect::<Result<_>>()?,
            dec: (0..n_layers).map(|i| DecoderLayer::new(&w, &format!("dec.layers.{i}"))).collect::<Result<_>>()?,
            bos: w.get("bos")?,
            step: w.get("step.weight")?,
            out: Linear::new(&w, "out")?,
        })
    }

    /// predictor.order_page: positions (0 = read first) for boxes in pixels of a
    /// `width` x `height` image; `labels` are raw rf-detr names; `fmap` [784, 256].
    pub fn order_page(
        &self,
        fmap: &Tensor,
        boxes: &[[f32; 4]],
        labels: &[&str],
        width: f32,
        height: f32,
    ) -> Result<Vec<usize>> {
        let n = boxes.len();
        if n == 0 {
            return Ok(vec![]);
        }
        if n == 1 {
            return Ok(vec![0]);
        }
        if n > MAX_BOXES {
            return Ok(raster_positions(boxes));
        }
        let b1000: Vec<[f32; 4]> = boxes
            .iter()
            .map(|b| {
                [
                    b[0] / width.max(1.0) * 1000.0,
                    b[1] / height.max(1.0) * 1000.0,
                    b[2] / width.max(1.0) * 1000.0,
                    b[3] / height.max(1.0) * 1000.0,
                ]
            })
            .collect();
        // canonical (y-banded raster) order: order[p] = original index at raster position p
        let band_h = 1000.0f32 / 24.0;
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            let ka = ((b1000[a][1] / band_h).floor() as i64, b1000[a][0], b1000[a][1], b1000[a][2]);
            let kb = ((b1000[b][1] / band_h).floor() as i64, b1000[b][0], b1000[b][1], b1000[b][2]);
            ka.partial_cmp(&kb).unwrap_or(std::cmp::Ordering::Equal)
        });
        let classes = layout_classes();
        let text = classes.iter().position(|c| *c == "Text").unwrap();
        let mut feats = Vec::with_capacity(n * 8);
        let mut labs = Vec::with_capacity(n);
        for &i in &order {
            let b = b1000[i];
            let (x0, y0, x1, y1) = (b[0] / 1000.0, b[1] / 1000.0, b[2] / 1000.0, b[3] / 1000.0);
            feats.extend_from_slice(&[x0, y0, x1, y1, (x0 + x1) / 2.0, (y0 + y1) / 2.0, x1 - x0, y1 - y0]);
            labs.push(classes.iter().position(|c| *c == labels[i]).unwrap_or(text) as u32);
        }
        let dev = Device::Cpu;
        let ft = Tensor::from_vec(feats, (n, 8), &dev)?;
        let li = Tensor::from_vec(labs, n, &dev)?;
        let mut x = (self.geom.fwd(&ft)? + self.lab.index_select(&li, 0)?)?;
        let f = self.feat_proj.fwd(fmap)?.broadcast_add(&self.feat_pos.narrow(0, 0, fmap.dim(0)?)?)?;
        for l in &self.ctx {
            x = l.fwd(&x, &f, None)?;
        }
        let mem = x; // [n, d]
        let mut used = vec![false; n];
        let mut din: Vec<Tensor> = vec![(&self.bos + self.step.get(0)?)?];
        let mut seq = Vec::with_capacity(n);
        for t in 0..n {
            let xin = Tensor::stack(&din, 0)?; // [t+1, d]
            let l = t + 1;
            let mut m = vec![0f32; l * l];
            for i in 0..l {
                for j in i + 1..l {
                    m[i * l + j] = f32::NEG_INFINITY;
                }
            }
            let mask = Tensor::from_vec(m, (l, l), &dev)?;
            let mut h = xin;
            for layer in &self.dec {
                h = layer.fwd(&h, &mem, Some(&mask))?;
            }
            let logits = self.out.fwd(&h.get(t)?.unsqueeze(0)?)?.squeeze(0)?.to_vec1::<f32>()?;
            let mut best = None;
            for (i, &v) in logits.iter().enumerate().take(n) {
                if used[i] {
                    continue;
                }
                if best.is_none_or(|(_, bv)| v > bv) {
                    best = Some((i, v));
                }
            }
            let nxt = best.map(|b| b.0).unwrap_or(0);
            seq.push(nxt);
            used[nxt] = true;
            din.push((mem.get(nxt)? + self.step.get((t + 1).min(MAX_BOXES))?)?);
        }
        let mut position = vec![0; n];
        for (rank, p) in seq.into_iter().enumerate() {
            position[order[p]] = rank;
        }
        Ok(position)
    }
}

pub fn raster_positions(boxes: &[[f32; 4]]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..boxes.len()).collect();
    idx.sort_by(|&a, &b| {
        (boxes[a][1], boxes[a][0]).partial_cmp(&(boxes[b][1], boxes[b][0])).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut pos = vec![0; boxes.len()];
    for (rank, i) in idx.into_iter().enumerate() {
        pos[i] = rank;
    }
    pos
}
