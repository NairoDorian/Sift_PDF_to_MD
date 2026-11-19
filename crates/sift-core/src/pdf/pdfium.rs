//! pdfium through its C API, loaded at run time (libloading), the way V0 used it through
//! pypdfium2 (same official pdfium-binaries build, `chromium/7891`, so the same characters,
//! boxes and pixels).
//!
//! pdfium is not thread-safe: every call runs under one global lock. Pages and text pages
//! are closed explicitly, in the order pypdfium2 closes them (form env first); V0 crashed
//! once when Python's garbage collector closed a page at a random moment.

use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use anyhow::{Context, Result, anyhow, bail};
use libloading::Library;

pub type Doc = *mut c_void;
pub type Page = *mut c_void;
pub type TextPage = *mut c_void;
pub type Bitmap = *mut c_void;
pub type FormHandle = *mut c_void;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct RectF {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct SizeF {
    width: f32,
    height: f32,
}

/// FPDF_FORMFILLINFO, version 2, every callback null (pypdfium2's default config). pdfium
/// only reads callbacks it finds non-null; the block is zero-filled and generously sized.
#[repr(C)]
struct FormFillInfo {
    version: c_int,
    _rest: [usize; 63],
}

pub const FPDFBITMAP_BGR: c_int = 2;
pub const FLAT_NORMALDISPLAY: c_int = 0;
pub const FORMTYPE_NONE: c_int = 0;

macro_rules! api {
    ($($name:ident : fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        #[allow(non_snake_case)]
        pub struct Api {
            _lib: Library,
            $(pub $name: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }
        impl Api {
            unsafe fn load(lib: Library) -> Result<Api> {
                unsafe {
                    Ok(Api {
                        $($name: *lib.get(concat!(stringify!($name), "\0").as_bytes())
                            .with_context(|| format!("pdfium has no {}", stringify!($name)))?,)*
                        _lib: lib,
                    })
                }
            }
        }
    };
}

api! {
    FPDF_InitLibrary: fn();
    FPDF_GetLastError: fn() -> c_ulong;
    FPDF_LoadMemDocument64: fn(*const c_void, usize, *const c_char) -> Doc;
    FPDF_CloseDocument: fn(Doc);
    FPDF_GetPageCount: fn(Doc) -> c_int;
    FPDF_GetPageSizeByIndexF: fn(Doc, c_int, *mut SizeF) -> c_int;
    FPDF_LoadPage: fn(Doc, c_int) -> Page;
    FPDF_ClosePage: fn(Page);
    FPDF_GetPageWidthF: fn(Page) -> f32;
    FPDF_GetPageHeightF: fn(Page) -> f32;
    FPDF_GetPageBoundingBox: fn(Page, *mut RectF) -> c_int;
    FPDFPage_GetRotation: fn(Page) -> c_int;
    FPDFPage_Flatten: fn(Page, c_int) -> c_int;
    FPDF_GetFormType: fn(Doc) -> c_int;
    FPDFDOC_InitFormFillEnvironment: fn(Doc, *mut FormFillInfo) -> FormHandle;
    FPDFDOC_ExitFormFillEnvironment: fn(FormHandle);
    FORM_OnAfterLoadPage: fn(Page, FormHandle);
    FORM_OnBeforeClosePage: fn(Page, FormHandle);
    FPDF_FFLDraw: fn(FormHandle, Bitmap, Page, c_int, c_int, c_int, c_int, c_int, c_int);
    FPDFText_LoadPage: fn(Page) -> TextPage;
    FPDFText_ClosePage: fn(TextPage);
    FPDFText_CountChars: fn(TextPage) -> c_int;
    FPDFText_GetUnicode: fn(TextPage, c_int) -> c_uint;
    FPDFText_GetCharAngle: fn(TextPage, c_int) -> f32;
    FPDFText_GetLooseCharBox: fn(TextPage, c_int, *mut RectF) -> c_int;
    FPDFText_GetCharBox: fn(TextPage, c_int, *mut f64, *mut f64, *mut f64, *mut f64) -> c_int;
    FPDFText_GetFontInfo: fn(TextPage, c_int, *mut c_void, c_ulong, *mut c_int) -> c_ulong;
    FPDFText_GetFontSize: fn(TextPage, c_int) -> f64;
    FPDFText_GetFontWeight: fn(TextPage, c_int) -> c_int;
    FPDFBitmap_CreateEx: fn(c_int, c_int, c_int, *mut c_void, c_int) -> Bitmap;
    FPDFBitmap_FillRect: fn(Bitmap, c_int, c_int, c_int, c_int, c_ulong) -> c_int;
    FPDFBitmap_GetBuffer: fn(Bitmap) -> *mut c_void;
    FPDFBitmap_GetStride: fn(Bitmap) -> c_int;
    FPDFBitmap_Destroy: fn(Bitmap);
    FPDF_RenderPageBitmap: fn(Bitmap, Page, c_int, c_int, c_int, c_int, c_int, c_int);
}

// SAFETY: every use of the function pointers happens under LOCK.
unsafe impl Send for Api {}
unsafe impl Sync for Api {}

static API: OnceLock<std::result::Result<Api, String>> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());
static LIB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Where to load pdfium from (call before the first document). Default: `SIFT_PDFIUM`,
/// else `pdfium.dll` next to the executable, else the runtime folder of the workspace.
pub fn set_library_path(p: PathBuf) {
    let _ = LIB_PATH.set(p);
}

pub fn default_library_candidates() -> Vec<PathBuf> {
    let name = if cfg!(windows) {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };
    let mut v = Vec::new();
    if let Some(p) = LIB_PATH.get() {
        v.push(p.clone());
    }
    if let Ok(p) = std::env::var("SIFT_PDFIUM") {
        v.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            v.push(d.join(name));
            // dev builds: target/<profile>/sift.exe -> <repository>/runtime/pdfium/
            for anc in d.ancestors().take(5) {
                v.push(anc.join("runtime").join("pdfium").join(name));
            }
        }
    }
    v
}

fn api() -> Result<&'static Api> {
    let r = API.get_or_init(|| {
        let mut errors = Vec::new();
        for p in default_library_candidates() {
            if !p.exists() {
                continue;
            }
            match unsafe { Library::new(&p) } {
                Ok(lib) => match unsafe { Api::load(lib) } {
                    Ok(api) => {
                        unsafe { (api.FPDF_InitLibrary)() };
                        log::info!("pdfium loaded from {}", p.display());
                        return Ok(api);
                    }
                    Err(e) => errors.push(format!("{}: {e}", p.display())),
                },
                Err(e) => errors.push(format!("{}: {e}", p.display())),
            }
        }
        Err(format!("pdfium library not found (run `sift setup pdfium`). Tried: {}", errors.join("; ")))
    });
    r.as_ref().map_err(|e| anyhow!(e.clone()))
}

/// The global pdfium lock: hold it for every sequence of pdfium calls.
pub struct Guard<'a> {
    _g: MutexGuard<'a, ()>,
    pub api: &'static Api,
}

pub fn lock() -> Result<Guard<'static>> {
    let api = api()?;
    let g = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    Ok(Guard { _g: g, api })
}

/// One open PDF. `flatten`: init the form env and flatten every page on first use, as
/// V0's PdfDocument and pdftext do (forms and annotations become page content).
pub struct Document {
    raw: Doc,
    form: FormHandle,
    _form_info: Option<Box<FormFillInfo>>,
    _bytes: Vec<u8>,
    pub page_count: usize,
    pub page_sizes: Vec<(f64, f64)>,
    flatten: bool,
    flattened: Mutex<Vec<bool>>,
    pub path: PathBuf,
}

unsafe impl Send for Document {}
unsafe impl Sync for Document {}

impl Document {
    pub fn open(path: &Path, flatten: bool) -> Result<Document> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let g = lock()?;
        let a = g.api;
        unsafe {
            let raw = (a.FPDF_LoadMemDocument64)(bytes.as_ptr() as *const c_void, bytes.len(), std::ptr::null());
            if raw.is_null() {
                bail!("pdfium could not open {} (error {})", path.display(), (a.FPDF_GetLastError)());
            }
            let mut form: FormHandle = std::ptr::null_mut();
            let mut info = None;
            if flatten && (a.FPDF_GetFormType)(raw) != FORMTYPE_NONE {
                let mut fi = Box::new(FormFillInfo { version: 2, _rest: [0; 63] });
                form = (a.FPDFDOC_InitFormFillEnvironment)(raw, &mut *fi);
                if form.is_null() {
                    (a.FPDF_CloseDocument)(raw);
                    bail!("initialising the form environment failed for {}", path.display());
                }
                info = Some(fi);
            }
            let n = (a.FPDF_GetPageCount)(raw).max(0) as usize;
            let mut sizes = Vec::with_capacity(n);
            for i in 0..n {
                let mut s = SizeF::default();
                (a.FPDF_GetPageSizeByIndexF)(raw, i as c_int, &mut s);
                sizes.push((s.width as f64, s.height as f64));
            }
            Ok(Document {
                raw,
                form,
                _form_info: info,
                _bytes: bytes,
                page_count: n,
                page_sizes: sizes,
                flatten,
                flattened: Mutex::new(vec![false; n]),
                path: path.to_path_buf(),
            })
        }
    }

    unsafe fn load_page(&self, a: &Api, idx: usize) -> Result<Page> {
        unsafe {
            let p = (a.FPDF_LoadPage)(self.raw, idx as c_int);
            if p.is_null() {
                bail!("pdfium could not load page {}", idx + 1);
            }
            if !self.form.is_null() {
                (a.FORM_OnAfterLoadPage)(p, self.form);
            }
            Ok(p)
        }
    }

    unsafe fn close_page(&self, a: &Api, p: Page) {
        unsafe {
            if !self.form.is_null() {
                (a.FORM_OnBeforeClosePage)(p, self.form);
            }
            (a.FPDF_ClosePage)(p);
        }
    }

    /// The page, flattened once (pdftext.pdf.utils.flatten, then closed and re-loaded).
    unsafe fn page(&self, a: &Api, idx: usize) -> Result<Page> {
        unsafe {
            let mut p = self.load_page(a, idx)?;
            if self.flatten {
                let mut fl = self.flattened.lock().unwrap();
                if !fl[idx] {
                    let rc = (a.FPDFPage_Flatten)(p, FLAT_NORMALDISPLAY);
                    if rc == 0 {
                        // FLATTEN_FAIL: pdftext raises; V0 would have failed the file
                        self.close_page(a, p);
                        bail!("failed to flatten annotations / form fields on page {}", idx + 1);
                    }
                    fl[idx] = true;
                    self.close_page(a, p);
                    p = self.load_page(a, idx)?;
                }
            }
            Ok(p)
        }
    }

    /// Run `f` with the (flattened) page, under the pdfium lock; the page is closed after.
    pub fn with_page<T>(&self, idx: usize, f: impl FnOnce(&Api, Page) -> Result<T>) -> Result<T> {
        let g = lock()?;
        unsafe {
            let p = self.page(g.api, idx)?;
            let r = f(g.api, p);
            self.close_page(g.api, p);
            r
        }
    }

    /// Render at `scale` (= dpi/72) as pypdfium2's PdfPage.render(scale, draw_annots=False):
    /// BGR bitmap of ceil(w*scale) x ceil(h*scale), filled white, no flags, forms drawn
    /// with FPDF_FFLDraw when a form env exists. Returns RGB bytes.
    pub fn render_rgb(&self, idx: usize, scale: f64) -> Result<crate::imaging::Rgb> {
        let form = self.form;
        self.with_page(idx, |a, p| unsafe {
            let pw = (a.FPDF_GetPageWidthF)(p) as f64;
            let ph = (a.FPDF_GetPageHeightF)(p) as f64;
            let w = (pw * scale).ceil() as c_int;
            let h = (ph * scale).ceil() as c_int;
            if w < 1 || h < 1 {
                bail!("page {} has no size", idx + 1);
            }
            let bmp = (a.FPDFBitmap_CreateEx)(w, h, FPDFBITMAP_BGR, std::ptr::null_mut(), 0);
            if bmp.is_null() {
                bail!("pdfium could not allocate a {w}x{h} bitmap");
            }
            (a.FPDFBitmap_FillRect)(bmp, 0, 0, w, h, 0xFFFF_FFFF);
            (a.FPDF_RenderPageBitmap)(bmp, p, 0, 0, w, h, 0, 0);
            if !form.is_null() {
                (a.FPDF_FFLDraw)(form, bmp, p, 0, 0, w, h, 0, 0);
            }
            let stride = (a.FPDFBitmap_GetStride)(bmp) as usize;
            let buf = (a.FPDFBitmap_GetBuffer)(bmp) as *const u8;
            let (wu, hu) = (w as usize, h as usize);
            let mut rgb = vec![0u8; wu * hu * 3];
            for y in 0..hu {
                let row = std::slice::from_raw_parts(buf.add(y * stride), wu * 3);
                let out = &mut rgb[y * wu * 3..(y + 1) * wu * 3];
                for x in 0..wu {
                    out[x * 3] = row[x * 3 + 2];
                    out[x * 3 + 1] = row[x * 3 + 1];
                    out[x * 3 + 2] = row[x * 3];
                }
            }
            (a.FPDFBitmap_Destroy)(bmp);
            Ok(crate::imaging::Rgb { width: wu, height: hu, data: rgb })
        })
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        if let Ok(g) = lock() {
            unsafe {
                if !self.form.is_null() {
                    (g.api.FPDFDOC_ExitFormFillEnvironment)(self.form);
                }
                (g.api.FPDF_CloseDocument)(self.raw);
            }
        }
    }
}

/// Raw per-character data of one page, as pdftext's get_chars reads it.
pub struct RawChar {
    pub code: u32,
    pub angle: f32,
    /// l, b, r, t as returned (loose: f32 widened; tight: f64)
    pub l: f64,
    pub b: f64,
    pub r: f64,
    pub t: f64,
    pub font_name: String,
    pub font_flags: i32,
    pub font_size: f64,
    pub font_weight: i32,
}

pub struct RawPage {
    /// FPDF_GetPageBoundingBox: left, bottom, right, top (f32 widened)
    pub bbox: [f64; 4],
    pub rotation: i32,
    pub chars: Vec<RawChar>,
}

impl Document {
    /// pdftext.pdf.chars.get_chars's pdfium reads (quote_loosebox as given).
    pub fn raw_chars(&self, idx: usize, quote_loosebox: bool) -> Result<RawPage> {
        self.with_page(idx, |a, p| unsafe {
            let tp = (a.FPDFText_LoadPage)(p);
            if tp.is_null() {
                bail!("failed to load text page {}", idx + 1);
            }
            let mut rect = RectF::default();
            if (a.FPDF_GetPageBoundingBox)(p, &mut rect) == 0 {
                (a.FPDFText_ClosePage)(tp);
                bail!("failed to get page bounding box");
            }
            let bbox = [rect.left as f64, rect.bottom as f64, rect.right as f64, rect.top as f64];
            let rot = match (a.FPDFPage_GetRotation)(p) {
                1 => 90,
                2 => 180,
                3 => 270,
                _ => 0,
            };
            let n = (a.FPDFText_CountChars)(tp).max(0);
            let mut chars = Vec::with_capacity(n as usize);
            let mut fbuf = vec![0u8; 256];
            for i in 0..n {
                let mut code = (a.FPDFText_GetUnicode)(tp, i);
                if (0xD800..=0xDFFF).contains(&code) {
                    code = 0xFFFD;
                }
                let angle = (a.FPDFText_GetCharAngle)(tp, i);
                let loose = angle == 0.0 && (code != 39 || quote_loosebox);
                let (l, b, r, t);
                if loose {
                    let mut rc = RectF::default();
                    if (a.FPDFText_GetLooseCharBox)(tp, i, &mut rc) == 0 {
                        (a.FPDFText_ClosePage)(tp);
                        bail!("Failed to get charbox.");
                    }
                    l = rc.left as f64;
                    b = rc.bottom as f64;
                    r = rc.right as f64;
                    t = rc.top as f64;
                } else {
                    let (mut cl, mut cr, mut cb, mut ct) = (0f64, 0f64, 0f64, 0f64);
                    if (a.FPDFText_GetCharBox)(tp, i, &mut cl, &mut cr, &mut cb, &mut ct) == 0 {
                        (a.FPDFText_ClosePage)(tp);
                        bail!("Failed to get charbox.");
                    }
                    l = cl;
                    b = cb;
                    r = cr;
                    t = ct;
                }
                let mut flags: c_int = 0;
                let mut len = (a.FPDFText_GetFontInfo)(
                    tp,
                    i,
                    fbuf.as_mut_ptr() as *mut c_void,
                    fbuf.len() as c_ulong,
                    &mut flags,
                );
                if len as usize > fbuf.len() {
                    fbuf = vec![0u8; len as usize];
                    len = (a.FPDFText_GetFontInfo)(
                        tp,
                        i,
                        fbuf.as_mut_ptr() as *mut c_void,
                        fbuf.len() as c_ulong,
                        &mut flags,
                    );
                }
                let (name, fl) = if len > 0 {
                    let end = fbuf.iter().position(|&c| c == 0).unwrap_or(fbuf.len());
                    (String::from_utf8_lossy(&fbuf[..end]).into_owned(), flags)
                } else {
                    (String::new(), 0)
                };
                chars.push(RawChar {
                    code,
                    angle,
                    l,
                    b,
                    r,
                    t,
                    font_name: name,
                    font_flags: fl,
                    font_size: (a.FPDFText_GetFontSize)(tp, i),
                    font_weight: (a.FPDFText_GetFontWeight)(tp, i),
                });
            }
            (a.FPDFText_ClosePage)(tp);
            Ok(RawPage { bbox, rotation: rot, chars })
        })
    }
}
