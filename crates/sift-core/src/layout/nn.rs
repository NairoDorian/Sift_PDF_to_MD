//! Small neural-network helpers on candle (CPU), for the layout models.
//!
//! Only what rf-detr and the reading-order head need: linear layers, layer norm, multi-head
//! attention (PyTorch's packed in_proj layout), MLPs. Weights come straight from the vendor's
//! PyTorch checkpoints (candle reads .pth zips), so nothing is converted at setup.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use candle_core::{D, DType, Device, Tensor};

pub struct Weights {
    pub map: HashMap<String, Tensor>,
}

impl Weights {
    pub fn load_pth(path: &Path, key: Option<&str>) -> Result<Weights> {
        let all =
            candle_core::pickle::read_all_with_key(path, key).with_context(|| format!("reading {}", path.display()))?;
        let mut map = HashMap::new();
        for (k, t) in all {
            let t = if t.dtype() == DType::F32 { t } else { t.to_dtype(DType::F32)? };
            map.insert(k, t);
        }
        Ok(Weights { map })
    }

    /// The same weights on another device.
    pub fn on(&self, dev: &Device) -> Result<Weights> {
        if dev.is_cpu() {
            return Ok(Weights { map: self.map.clone() });
        }
        Ok(Weights { map: self.map.iter().map(|(k, v)| Ok((k.clone(), v.to_device(dev)?))).collect::<Result<_>>()? })
    }

    pub fn get(&self, name: &str) -> Result<Tensor> {
        self.map.get(name).cloned().ok_or_else(|| anyhow!("weight {name} missing from the checkpoint"))
    }
}

pub struct Linear {
    w_t: Tensor, // [in, out]
    b: Option<Tensor>,
}

impl Linear {
    pub fn new(w: &Weights, prefix: &str) -> Result<Linear> {
        let wt = w.get(&format!("{prefix}.weight"))?.t()?.contiguous()?;
        let b = w.map.get(&format!("{prefix}.bias")).cloned();
        Ok(Linear { w_t: wt, b })
    }
    pub fn from_parts(weight: Tensor, bias: Option<Tensor>) -> Result<Linear> {
        Ok(Linear { w_t: weight.t()?.contiguous()?, b: bias })
    }
    /// x: [..., in] -> [..., out]
    pub fn fwd(&self, x: &Tensor) -> Result<Tensor> {
        let dims = x.dims().to_vec();
        let inn = *dims.last().unwrap();
        let rows: usize = dims[..dims.len() - 1].iter().product();
        let y = x.reshape((rows, inn))?.matmul(&self.w_t)?;
        let y = match &self.b {
            Some(b) => y.broadcast_add(b)?,
            None => y,
        };
        let mut od = dims.clone();
        *od.last_mut().unwrap() = self.w_t.dim(1)?;
        Ok(y.reshape(od)?)
    }
}

pub struct LayerNorm {
    w: Tensor,
    b: Tensor,
    eps: f32,
}

impl LayerNorm {
    pub fn new(w: &Weights, prefix: &str, eps: f32) -> Result<LayerNorm> {
        Ok(LayerNorm { w: w.get(&format!("{prefix}.weight"))?, b: w.get(&format!("{prefix}.bias"))?, eps })
    }
    pub fn fwd(&self, x: &Tensor) -> Result<Tensor> {
        Ok(candle_nn::ops::layer_norm(&x.contiguous()?, &self.w, &self.b, self.eps)?)
    }
}

/// MLP of n linear layers with ReLU between them (rf-detr's `MLP`).
pub struct Mlp {
    layers: Vec<Linear>,
}

impl Mlp {
    pub fn new(w: &Weights, prefix: &str, n: usize) -> Result<Mlp> {
        Ok(Mlp { layers: (0..n).map(|i| Linear::new(w, &format!("{prefix}.layers.{i}"))).collect::<Result<_>>()? })
    }
    pub fn fwd(&self, x: &Tensor) -> Result<Tensor> {
        let mut x = x.clone();
        let n = self.layers.len();
        for (i, l) in self.layers.iter().enumerate() {
            x = l.fwd(&x)?;
            if i + 1 < n {
                x = x.relu()?;
            }
        }
        Ok(x)
    }
}

/// torch.nn.MultiheadAttention (batch_first, packed in_proj), inference only.
pub struct Mha {
    q: Linear,
    k: Linear,
    v: Linear,
    out: Linear,
    heads: usize,
}

impl Mha {
    pub fn new(w: &Weights, prefix: &str, heads: usize) -> Result<Mha> {
        let ipw = w.get(&format!("{prefix}.in_proj_weight"))?;
        let ipb = w.get(&format!("{prefix}.in_proj_bias"))?;
        let d = ipw.dim(1)?;
        let part = |i: usize| -> Result<Linear> {
            Linear::from_parts(ipw.narrow(0, i * d, d)?, Some(ipb.narrow(0, i * d, d)?))
        };
        Ok(Mha { q: part(0)?, k: part(1)?, v: part(2)?, out: Linear::new(w, &format!("{prefix}.out_proj"))?, heads })
    }

    /// q: [Lq, d], k/v: [Lk, d]; `mask`: optional [Lq, Lk] additive (0 / -inf).
    pub fn fwd(&self, q: &Tensor, k: &Tensor, v: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let q = self.q.fwd(q)?;
        let k = self.k.fwd(k)?;
        let v = self.v.fwd(v)?;
        let o = attention(&q, &k, &v, self.heads, mask)?;
        self.out.fwd(&o)
    }
}

/// softmax(q k^T / sqrt(dh)) v over `heads` heads. q: [Lq, d], k, v: [Lk, d] -> [Lq, d].
pub fn attention(q: &Tensor, k: &Tensor, v: &Tensor, heads: usize, mask: Option<&Tensor>) -> Result<Tensor> {
    let (lq, d) = q.dims2()?;
    let lk = k.dim(0)?;
    let dh = d / heads;
    let qh = q.reshape((lq, heads, dh))?.transpose(0, 1)?.contiguous()?; // [h, Lq, dh]
    let kh = k.reshape((lk, heads, dh))?.transpose(0, 1)?.contiguous()?;
    let vh = v.reshape((lk, heads, dh))?.transpose(0, 1)?.contiguous()?;
    let scale = 1.0 / (dh as f64).sqrt();
    let mut s = (qh.matmul(&kh.t()?.contiguous()?)? * scale)?; // [h, Lq, Lk]
    if let Some(m) = mask {
        s = s.broadcast_add(m)?;
    }
    let p = candle_nn::ops::softmax_last_dim(&s)?;
    let o = p.matmul(&vh)?; // [h, Lq, dh]
    Ok(o.transpose(0, 1)?.contiguous()?.reshape((lq, d))?)
}

pub fn cpu() -> Device {
    Device::Cpu
}

pub fn last_dim_max(x: &Tensor) -> Result<Tensor> {
    Ok(x.max(D::Minus1)?)
}
