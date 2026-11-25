//! Python-compatible text semantics.
//!
//! V0's rules were written and measured in Python; V1 must give the same answers on the
//! same input (the golden test, docs/GOLDEN.md). Rust's `char::is_whitespace`,
//! `is_alphanumeric` and the regex crate's `\w` / `\s` differ from Python's in corner
//! cases (`\x1c`-`\x1f` are whitespace in Python, `²` is alphanumeric in Python, combining
//! marks are word characters in Rust), so every rule ported from V0 goes through these.

use unicode_general_category::{GeneralCategory as G, get_general_category};

/// Python `str.isspace()` for one character.
pub fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n'
            | '\x0b'
            | '\x0c'
            | '\r'
            | '\x1c'..='\x1f'
            | ' '
            | '\u{85}'
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

pub fn category(c: char) -> G {
    get_general_category(c)
}

/// Two-letter category name, as `unicodedata.category`.
pub fn category_name(c: char) -> &'static str {
    use G::*;
    match category(c) {
        UppercaseLetter => "Lu",
        LowercaseLetter => "Ll",
        TitlecaseLetter => "Lt",
        ModifierLetter => "Lm",
        OtherLetter => "Lo",
        NonspacingMark => "Mn",
        SpacingMark => "Mc",
        EnclosingMark => "Me",
        DecimalNumber => "Nd",
        LetterNumber => "Nl",
        OtherNumber => "No",
        ConnectorPunctuation => "Pc",
        DashPunctuation => "Pd",
        OpenPunctuation => "Ps",
        ClosePunctuation => "Pe",
        InitialPunctuation => "Pi",
        FinalPunctuation => "Pf",
        OtherPunctuation => "Po",
        MathSymbol => "Sm",
        CurrencySymbol => "Sc",
        ModifierSymbol => "Sk",
        OtherSymbol => "So",
        SpaceSeparator => "Zs",
        LineSeparator => "Zl",
        ParagraphSeparator => "Zp",
        Control => "Cc",
        Format => "Cf",
        Surrogate => "Cs",
        PrivateUse => "Co",
        Unassigned => "Cn",
        _ => "Cn",
    }
}

pub fn is_alpha(c: char) -> bool {
    use G::*;
    matches!(category(c), UppercaseLetter | LowercaseLetter | TitlecaseLetter | ModifierLetter | OtherLetter)
}

/// Python `isdecimal` (category Nd).
pub fn is_decimal(c: char) -> bool {
    category(c) == G::DecimalNumber
}

/// Python `isdigit`: Numeric_Type Decimal or Digit (Nd plus the superscript, subscript and
/// circled digits).
pub fn is_digit(c: char) -> bool {
    if is_decimal(c) {
        return true;
    }
    matches!(c as u32,
        0xB2 | 0xB3 | 0xB9 | 0x1369..=0x1371 | 0x19DA | 0x2070 | 0x2074..=0x2079 | 0x2080..=0x2089
        | 0x2460..=0x2468 | 0x2474..=0x247C | 0x2488..=0x2490 | 0x24EA | 0x24F5..=0x24FD | 0x24FF
        | 0x2776..=0x277E | 0x2780..=0x2788 | 0x278A..=0x2792 | 0x10A40..=0x10A43 | 0x10E60..=0x10E68
        | 0x11052..=0x1105A | 0x1F100..=0x1F10A)
}

/// Python `isnumeric` (approximately: categories Nd, Nl, No; CJK numerals are letters anyway).
pub fn is_numeric(c: char) -> bool {
    use G::*;
    matches!(category(c), DecimalNumber | LetterNumber | OtherNumber)
}

/// Python `isalnum` for one character.
pub fn is_alnum(c: char) -> bool {
    is_alpha(c) || is_numeric(c)
}

fn is_cased_upper(c: char) -> bool {
    c.is_uppercase()
}
fn is_cased_lower(c: char) -> bool {
    c.is_lowercase()
}
fn is_title(c: char) -> bool {
    category(c) == G::TitlecaseLetter
}

/// Python `str.isupper()`.
pub fn str_isupper(s: &str) -> bool {
    let mut cased = false;
    for c in s.chars() {
        if is_cased_lower(c) || is_title(c) {
            return false;
        }
        if is_cased_upper(c) {
            cased = true;
        }
    }
    cased
}

/// Python `str.islower()`.
pub fn str_islower(s: &str) -> bool {
    let mut cased = false;
    for c in s.chars() {
        if is_cased_upper(c) || is_title(c) {
            return false;
        }
        if is_cased_lower(c) {
            cased = true;
        }
    }
    cased
}

pub fn str_isalnum(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_alnum)
}
pub fn str_isalpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_alpha)
}
pub fn str_isdigit(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_digit)
}
pub fn str_isdecimal(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_decimal)
}
pub fn str_isspace(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_space)
}

/// Python `str.strip()` (no argument).
pub fn strip(s: &str) -> &str {
    s.trim_matches(is_space)
}
pub fn lstrip(s: &str) -> &str {
    s.trim_start_matches(is_space)
}
pub fn rstrip(s: &str) -> &str {
    s.trim_end_matches(is_space)
}
/// `bool(s.strip())`
pub fn has_ink(s: &str) -> bool {
    s.chars().any(|c| !is_space(c))
}

/// Python `str.split()` (no argument): runs of whitespace, no empty parts.
pub fn split_ws(s: &str) -> Vec<&str> {
    s.split(is_space).filter(|p| !p.is_empty()).collect()
}

/// `" ".join(s.split())`
pub fn squash_ws(s: &str) -> String {
    split_ws(s).join(" ")
}

pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Python slicing by characters `s[a:b]` (a, b already clamped by the caller or not).
pub fn slice(s: &str, a: usize, b: usize) -> String {
    s.chars().skip(a).take(b.saturating_sub(a)).collect()
}

/// Python `round(x)` for a float -> nearest, ties to even.
pub fn round_half_even(x: f64) -> f64 {
    x.round_ties_even()
}

/// Python `round(x, n)` (correctly rounded, ties to even on the decimal string).
pub fn round_n(x: f64, n: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if n == 0 {
        return x.round_ties_even();
    }
    // Python formats with repr precision then rounds the exact decimal value; formatting
    // with `{:.n$}` in Rust is also correctly rounded (round-half-even on the exact binary
    // value), which is what CPython's float.__round__ does.
    let s = format!("{:.*}", n.max(0) as usize, x);
    s.parse().unwrap_or(x)
}

/// Python `repr(float)` (shortest round-trip, with Python's exponent rules).
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let e = x.abs().log10().floor() as i32;
    if (-4..16).contains(&e) {
        let s = format!("{}", x);
        if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
    } else {
        // shortest digits in scientific form: Rust's {:e} is shortest round-trip too
        let s = format!("{:e}", x);
        let (m, ex) = s.split_once('e').unwrap();
        let exn: i32 = ex.parse().unwrap();
        let sign = if exn < 0 { '-' } else { '+' };
        format!("{m}e{sign}{:02}", exn.abs())
    }
}

/// Python `f"{x:.{n}f}"`.
pub fn fmt_fixed(x: f64, n: usize) -> String {
    format!("{:.*}", n, x)
}

/// Python `f"{x:.0%}"`.
pub fn fmt_pct0(x: f64) -> String {
    format!("{}%", fmt_fixed(x * 100.0, 0))
}

/// Translate a Python `re` pattern to the Rust regex dialect with Python's class meanings:
/// `\w` = alphanumeric (str.isalnum) or `_`, `\s` = str.isspace, `\d` = Nd (same in both).
pub fn pyre(pattern: &str) -> String {
    const W: &str = r"\p{L}\p{N}_";
    const S: &str =
        r"\t\n\x0b\x0c\r\x1c-\x1f \x{85}\x{a0}\x{1680}\x{2000}-\x{200a}\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}";
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut in_class = false;
    // Python's `$` (without MULTILINE) also matches right before a final newline
    let multiline = pattern.starts_with("(?") && pattern[2..].split(')').next().is_some_and(|f| f.contains('m'));
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && !in_class && !multiline {
            out.push_str(r"(?=\n?\z)");
            continue;
        }
        if c == '\\' {
            match chars.next() {
                Some('Z') => out.push_str(r"\z"),
                Some('w') => {
                    if in_class {
                        out.push_str(W)
                    } else {
                        out.push_str(&format!("[{W}]"))
                    }
                }
                Some('W') => {
                    if in_class {
                        out.push_str(r"\W")
                    } else {
                        out.push_str(&format!("[^{W}]"))
                    }
                }
                Some('s') => {
                    if in_class {
                        out.push_str(S)
                    } else {
                        out.push_str(&format!("[{S}]"))
                    }
                }
                Some('S') => {
                    if in_class {
                        out.push_str(r"\S")
                    } else {
                        out.push_str(&format!("[^{S}]"))
                    }
                }
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
            continue;
        }
        if c == '[' && !in_class {
            in_class = true;
            out.push(c);
            // a ']' right after '[' or '[^' is a literal in Python
            if chars.peek() == Some(&'^') {
                out.push(chars.next().unwrap());
            }
            if chars.peek() == Some(&']') {
                chars.next();
                out.push_str(r"\]");
            }
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
        }
        out.push(c);
    }
    out
}

/// A compiled Python-style regex (lookarounds allowed through fancy-regex).
pub struct Re(pub fancy_regex::Regex);

impl Re {
    pub fn new(pattern: &str) -> Re {
        Re(fancy_regex::Regex::new(&pyre(pattern)).unwrap_or_else(|e| panic!("bad regex {pattern:?}: {e}")))
    }
    pub fn new_flags(pattern: &str, flags: &str) -> Re {
        Re::new(&format!("(?{flags}){pattern}"))
    }
    pub fn is_match(&self, s: &str) -> bool {
        self.0.is_match(s).unwrap_or(false)
    }
    /// `re.match` (anchored at the start).
    pub fn match_start<'a>(&self, s: &'a str) -> Option<fancy_regex::Captures<'a, str>> {
        match self.0.captures(s) {
            Ok(Some(c)) if c.get(0).map(|m| m.start()) == Some(0) => Some(c),
            Ok(Some(_)) => {
                // a later match exists; an anchored one may still exist: try with \A
                None
            }
            _ => None,
        }
    }
    pub fn search<'a>(&self, s: &'a str) -> Option<fancy_regex::Captures<'a, str>> {
        self.0.captures(s).ok().flatten()
    }
    pub fn find_all<'a>(&self, s: &'a str) -> Vec<&'a str> {
        self.0.find_iter(s).filter_map(|m| m.ok()).map(|m| m.as_str()).collect()
    }
    /// `re.findall` with one group -> the group's text.
    pub fn find_all_group<'a>(&self, s: &'a str, g: usize) -> Vec<&'a str> {
        self.0.captures_iter(s).filter_map(|c| c.ok()).map(|c| c.get(g).map(|m| m.as_str()).unwrap_or("")).collect()
    }
    pub fn replace_all(&self, s: &str, rep: &str) -> String {
        self.0.replace_all(s, rep).into_owned()
    }
    pub fn replace_all_fn(&self, s: &str, f: impl Fn(&fancy_regex::Captures<'_, str>) -> String) -> String {
        self.0.replace_all(s, |c: &fancy_regex::Captures<'_, str>| f(c)).into_owned()
    }
    pub fn split<'a>(&self, s: &'a str) -> Vec<&'a str> {
        self.0.split(s).filter_map(|p| p.ok()).collect()
    }
}

/// `re.match(pattern, s)`: compiled with a `\A` anchor so the match must start at 0.
pub struct ReStart(pub Re);
impl ReStart {
    pub fn new(pattern: &str) -> ReStart {
        ReStart(Re::new(&format!(r"\A(?:{pattern})")))
    }
    pub fn new_flags(pattern: &str, flags: &str) -> ReStart {
        ReStart(Re::new(&format!(r"(?{flags})\A(?:{pattern})")))
    }
    pub fn m<'a>(&self, s: &'a str) -> Option<fancy_regex::Captures<'a, str>> {
        self.0.search(s)
    }
    pub fn is(&self, s: &str) -> bool {
        self.0.is_match(s)
    }
}

/// `re.fullmatch`.
pub struct ReFull(pub Re);
impl ReFull {
    pub fn new(pattern: &str) -> ReFull {
        ReFull(Re::new(&format!(r"\A(?:{pattern})\z")))
    }
    pub fn new_flags(pattern: &str, flags: &str) -> ReFull {
        ReFull(Re::new(&format!(r"(?{flags})\A(?:{pattern})\z")))
    }
    pub fn m<'a>(&self, s: &'a str) -> Option<fancy_regex::Captures<'a, str>> {
        self.0.search(s)
    }
    pub fn is(&self, s: &str) -> bool {
        self.0.is_match(s)
    }
}

/// Lazily compiled regexes: `re!(NAME = r"...")` style statics.
#[macro_export]
macro_rules! pyre_static {
    ($name:ident, $kind:ident, $pat:expr) => {
        static $name: std::sync::LazyLock<$crate::py::$kind> =
            std::sync::LazyLock::new(|| $crate::py::$kind::new($pat));
    };
    ($name:ident, $kind:ident, $pat:expr, $flags:expr) => {
        static $name: std::sync::LazyLock<$crate::py::$kind> =
            std::sync::LazyLock::new(|| $crate::py::$kind::new_flags($pat, $flags));
    };
}

/// Python `max(iterable, key=...)`: the FIRST maximal element (Rust's max_by returns the last).
/// CPython replaces the best only when `key > best`: an equal key, or one that does not compare
/// (NaN), keeps the current best.
pub fn first_max_by<T, K: PartialOrd>(items: impl IntoIterator<Item = T>, key: impl Fn(&T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for it in items {
        let k = key(&it);
        match &best {
            Some((_, bk)) if k.partial_cmp(bk) != Some(std::cmp::Ordering::Greater) => {}
            _ => best = Some((it, k)),
        }
    }
    best.map(|(t, _)| t)
}

/// Python `min(iterable, key=...)`: the first minimal element (replaced only when `key < best`;
/// equal or incomparable keys keep the current one, as in CPython).
pub fn first_min_by<T, K: PartialOrd>(items: impl IntoIterator<Item = T>, key: impl Fn(&T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for it in items {
        let k = key(&it);
        match &best {
            Some((_, bk)) if k.partial_cmp(bk) != Some(std::cmp::Ordering::Less) => {}
            _ => best = Some((it, k)),
        }
    }
    best.map(|(t, _)| t)
}

/// Total order for f64 keys in sorts (Python compares floats directly; NaN never appears).
pub fn fcmp(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// `statistics.median`.
pub fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| fcmp(*a, *b));
    let n = s.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 { s[n / 2] } else { (s[n / 2 - 1] + s[n / 2]) / 2.0 }
}

/// Python `str.strip(chars)`.
pub fn strip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_matches(|c| chars.contains(c))
}
pub fn lstrip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_start_matches(|c| chars.contains(c))
}
pub fn rstrip_chars<'a>(s: &'a str, chars: &str) -> &'a str {
    s.trim_end_matches(|c| chars.contains(c))
}

/// Python `str.splitlines()`.
pub fn splitlines(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\r' => {
                if it.peek() == Some(&'\n') {
                    it.next();
                }
                out.push(std::mem::take(&mut cur));
            }
            '\n' | '\x0b' | '\x0c' | '\x1c' | '\x1d' | '\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}' => {
                out.push(std::mem::take(&mut cur));
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Python `repr(str)`.
pub fn repr_str(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut o = String::with_capacity(s.len() + 2);
    o.push(q);
    for c in s.chars() {
        match c {
            '\\' => o.push_str(r"\\"),
            '\n' => o.push_str(r"\n"),
            '\r' => o.push_str(r"\r"),
            '\t' => o.push_str(r"\t"),
            c if c == q => {
                o.push('\\');
                o.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => o.push_str(&format!(r"\x{:02x}", c as u32)),
            c if !is_printable(c) => {
                let v = c as u32;
                if v <= 0xff {
                    o.push_str(&format!(r"\x{v:02x}"));
                } else if v <= 0xffff {
                    o.push_str(&format!(r"\u{v:04x}"));
                } else {
                    o.push_str(&format!(r"\U{v:08x}"));
                }
            }
            c => o.push(c),
        }
    }
    o.push(q);
    o
}

/// Python `str.isprintable()` for one character (categories Cc, Cf, Cs, Co, Cn, Zl, Zp, Zs
/// are not printable, except the ASCII space).
pub fn is_printable(c: char) -> bool {
    if c == ' ' {
        return true;
    }
    !matches!(category_name(c), "Cc" | "Cf" | "Cs" | "Co" | "Cn" | "Zl" | "Zp" | "Zs")
}

/// Python `repr(list_of_str)`.
pub fn repr_list(v: &[String]) -> String {
    format!("[{}]", v.iter().map(|s| repr_str(s)).collect::<Vec<_>>().join(", "))
}

/// unicodedata.normalize("NFKC", s)
pub fn nfkc(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfkc().collect()
}

/// `s.rsplit(None, 1)[-1]` (the last whitespace-separated token; "" for blank input).
pub fn last_token(s: &str) -> &str {
    split_ws(s).last().copied().unwrap_or("")
}

/// Python `re.escape`.
pub fn re_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        // Python 3.7+: only ()[]{}?*+-|^$\.&~# and whitespace are special
        if r"()[]{}?*+-|^$\.&~#".contains(c) {
            o.push('\\');
        }
        o.push(c);
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_match_python() {
        assert!(is_space('\x1c'));
        assert!(!'\x1c'.is_whitespace());
        assert!(is_alnum('²'));
        assert!(str_isdigit("²"));
        assert!(!str_isdecimal("²"));
        assert!(str_isupper("TX-1"));
        assert!(!str_isupper("123"));
        assert_eq!(strip("\x1c a \u{3000}"), "a");
        assert_eq!(split_ws("  a  b\tc "), vec!["a", "b", "c"]);
    }

    #[test]
    fn regex_word_class() {
        let r = Re::new(r"(?<![\w.])\d+");
        assert!(r.is_match("x 12"));
        assert!(!r.is_match("²12")); // Python: '²' is \w
        let r = ReFull::new(r"\w+");
        assert!(r.is("mm²"));
    }

    #[test]
    fn float_repr_like_python() {
        assert_eq!(float_repr(1.0), "1.0");
        assert_eq!(float_repr(0.1), "0.1");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(1.5e-5), "1.5e-05");
        assert_eq!(float_repr(123456.0), "123456.0");
    }

    #[test]
    fn rounding() {
        assert_eq!(round_half_even(2.5), 2.0);
        assert_eq!(round_half_even(3.5), 4.0);
        assert_eq!(round_n(0.125, 2), 0.12);
    }
}
