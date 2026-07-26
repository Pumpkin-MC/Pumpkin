//! Physical sharding of oversized generated files.
//!
//! Some generated files (block.rs, biome.rs, ...) are hundreds of thousands of
//! lines. Instead of one giant file, they are written as a small stub
//! `X.rs` plus `X_parts/part_NNN.rs` files pulled in with `include!`, which is
//! purely textual, so compilation is equivalent to the single-file layout.
//!
//! Splitting only ever happens at complete-item boundaries:
//! 1. Top-level items are packed into parts referenced by top-level
//!    `include!("X_parts/part_NNN.rs");` lines in the stub.
//! 2. An oversized `mod name {` item keeps its open/close lines in the stub
//!    with indented `include!` lines between them (`include!` is valid in a
//!    module body).
//! 3. An oversized inherent `impl Type {` item is split into several parts,
//!    each a complete `impl Type { ... }` block, because `include!` is NOT
//!    valid in impl-item position (rustc: "non-impl item macro in impl item
//!    position"). Splitting an inherent impl into several inherent impls is
//!    semantically identical. Trait impls are never split.
//!
//! After splitting, the original content is reconstructed from the stub and
//! parts and compared byte-for-byte; any mismatch is a panic. This module is
//! the Rust twin of the one-off script that performed the initial split; keep
//! the boundary rules in sync so regeneration reproduces the same layout.

use std::fs;
use std::path::Path;

use crate::{OUT_DIR, write_file_if_changed};

/// Generated files that are written in the sharded stub + parts layout.
pub const SHARDED_FILES: &[&str] = &[
    "block.rs",
    "biome.rs",
    "tag.rs",
    "item.rs",
    "translation.rs",
    "advancement.rs",
];

/// Target maximum number of lines per part file.
const THRESHOLD: usize = 30000;

// ---------------------------------------------------------------------------
// Tokenizer: per-line state (brace depth at line start, in-comment/in-string)
// ---------------------------------------------------------------------------

/// For each line: (bracket depth at line start, line starts outside any
/// comment/string). Depth counts `{[(` minus `}])` outside strings, char
/// literals, and comments.
fn line_states(text: &str) -> Vec<(i64, bool)> {
    let b = text.as_bytes();
    let n = b.len();
    let mut states = vec![(0i64, true)];
    let mut depth = 0i64;
    // modes: 0 normal, 1 block comment (nesting), 2 string, 3 raw string
    let mut mode = 0u8;
    let mut comment_nest = 0usize;
    let mut raw_hashes = 0usize;
    let mut i = 0usize;
    while i < n {
        let mut c = b[i];
        if c == b'\n' {
            states.push((depth, mode == 0));
            i += 1;
            continue;
        }
        match mode {
            1 => {
                // Rust block comments nest.
                if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    comment_nest += 1;
                    i += 2;
                } else if c == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    comment_nest -= 1;
                    i += 2;
                    if comment_nest == 0 {
                        mode = 0;
                    }
                } else {
                    i += 1;
                }
                continue;
            }
            2 => {
                if c == b'\\' {
                    i += 2;
                } else {
                    if c == b'"' {
                        mode = 0;
                    }
                    i += 1;
                }
                continue;
            }
            3 => {
                if c == b'"' && b[i + 1..].len() >= raw_hashes
                    && b[i + 1..i + 1 + raw_hashes].iter().all(|&h| h == b'#')
                {
                    mode = 0;
                    i += 1 + raw_hashes;
                } else {
                    i += 1;
                }
                continue;
            }
            _ => {}
        }
        // mode == 0: normal code
        if c == b'/' && i + 1 < n {
            let nxt = b[i + 1];
            if nxt == b'/' {
                match text[i..].find('\n') {
                    Some(j) => {
                        i += j;
                        continue;
                    }
                    None => break,
                }
            }
            if nxt == b'*' {
                mode = 1;
                comment_nest = 1;
                i += 2;
                continue;
            }
        }
        if c == b'"' {
            mode = 2;
            i += 1;
            continue;
        }
        if c == b'r' || c == b'b' {
            let prev = if i > 0 { b[i - 1] } else { b' ' };
            let ident_tail = prev.is_ascii_alphanumeric() || prev == b'_';
            if !ident_tail {
                let mut j = i;
                if c == b'b' && j + 1 < n && matches!(b[j + 1], b'r' | b'"' | b'\'') {
                    if b[j + 1] == b'"' {
                        mode = 2;
                        i += 2;
                        continue;
                    }
                    if b[j + 1] == b'\'' {
                        // byte char literal: handle at the quote below
                        i += 1;
                        c = b'\'';
                    } else {
                        j += 1;
                    }
                }
                if c != b'\'' && j < n && b[j] == b'r' {
                    let mut k = j + 1;
                    let mut hashes = 0usize;
                    while k < n && b[k] == b'#' {
                        hashes += 1;
                        k += 1;
                    }
                    if k < n && b[k] == b'"' {
                        mode = 3;
                        raw_hashes = hashes;
                        i = k + 1;
                        continue;
                    }
                }
            }
            if c != b'\'' {
                i += 1;
                continue;
            }
        }
        if c == b'\'' {
            // char literal vs lifetime
            if i + 1 < n && b[i + 1] == b'\\' {
                let mut j = i + 2;
                while j < n && b[j] != b'\'' {
                    if b[j] == b'\\' {
                        j += 1;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            if i + 2 < n && b[i + 2] == b'\'' && b[i + 1] != b'\'' {
                i += 3; // 'x'
                continue;
            }
            i += 1; // lifetime
            continue;
        }
        match c {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => {
                depth -= 1;
                assert!(depth >= 0, "sharding: negative bracket depth at byte {i}");
            }
            _ => {}
        }
        i += 1;
    }
    states
}

// ---------------------------------------------------------------------------
// Boundary / unit computation
// ---------------------------------------------------------------------------

/// Leading lines that must stay in the stub (header comment, `#![..]`, `//!`).
fn is_header_line(line: &str) -> bool {
    let s = line.trim();
    if s.is_empty() {
        return true;
    }
    if let Some(rest) = s.strip_prefix("//") {
        // `///` outer docs belong to the next item; `//!` and `//` stay.
        return !rest.starts_with('/');
    }
    if s.starts_with("#![") {
        return true;
    }
    s.starts_with("/*") && s.ends_with("*/")
}

fn compute_prefix_len(lines: &[&str]) -> usize {
    let mut k = 0;
    while k < lines.len() && is_header_line(lines[k]) {
        k += 1;
    }
    while k > 0 && lines[k - 1].trim().is_empty() {
        k -= 1;
    }
    k
}

fn ends_item(line: &str) -> bool {
    let s = line.trim_end();
    s.ends_with(';') || s.ends_with('}')
}

/// Split `lines[start..end]` into runs of complete items at `base_depth`.
fn unit_boundaries(
    lines: &[&str],
    states: &[(i64, bool)],
    start: usize,
    end: usize,
    base_depth: i64,
) -> Vec<(usize, usize)> {
    let mut bounds = vec![start];
    let mut prev_nonblank: Option<&str> = None;
    for p in start..end {
        let (depth, clean) = states[p];
        if p > start
            && depth == base_depth
            && clean
            && let Some(prev) = prev_nonblank
            && ends_item(prev)
        {
            let s = prev.trim_start();
            if !(s.starts_with("//") || s.starts_with('#')) {
                bounds.push(p);
            }
        }
        if !lines[p].trim().is_empty() {
            prev_nonblank = Some(lines[p]);
        }
    }
    bounds.push(end);
    let mut units = Vec::new();
    for w in bounds.windows(2) {
        if w[1] > w[0] {
            units.push((w[0], w[1]));
        }
    }
    assert!(
        !units.is_empty() && units[0].0 == start && units[units.len() - 1].1 == end,
        "sharding: bad unit cover"
    );
    units
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `mod name {` optionally preceded by `pub` / `pub(...)`.
fn is_mod_open(line: &str) -> bool {
    let mut s = line;
    if let Some(rest) = s.strip_prefix("pub") {
        let rest = if let Some(r) = rest.strip_prefix('(') {
            match r.find(')') {
                Some(idx) => &r[idx + 1..],
                None => return false,
            }
        } else {
            rest
        };
        let Some(rest) = rest.strip_prefix(' ') else {
            return false;
        };
        s = rest.trim_start();
    }
    let Some(rest) = s.strip_prefix("mod ") else {
        return false;
    };
    let Some(body) = rest.trim_end().strip_suffix('{') else {
        return false;
    };
    is_ident(body.trim())
}

/// Inherent `impl Type {` (no generics, not a trait impl).
fn is_inherent_impl_open(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("impl ") else {
        return false;
    };
    let Some(body) = rest.trim_end().strip_suffix('{') else {
        return false;
    };
    is_ident(body.trim())
}

#[derive(PartialEq)]
enum ContainerKind {
    Mod,
    Impl,
}

/// If the unit is an oversized splittable container, return its kind and the
/// number of leading attribute/doc lines before the opening line.
fn container_kind(
    lines: &[&str],
    states: &[(i64, bool)],
    unit: (usize, usize),
    base_depth: i64,
) -> Option<(ContainerKind, usize)> {
    let (a, b) = unit;
    let mut head = a;
    while head < b {
        let s = lines[head].trim();
        if s.starts_with("#[") || s.starts_with("///") || s.is_empty() {
            head += 1;
        } else {
            break;
        }
    }
    if head >= b {
        return None;
    }
    let open_line = lines[head].trim();
    let close = lines[b - 1].trim_end_matches('\n');
    // closing line must be a lone, unindented `}` at the container's depth
    if close != "}" || states[b - 1].0 != base_depth + 1 {
        return None;
    }
    if is_mod_open(open_line) {
        return Some((ContainerKind::Mod, head - a));
    }
    if is_inherent_impl_open(open_line) && !open_line.contains(" for ") {
        return Some((ContainerKind::Impl, head - a));
    }
    None
}

// ---------------------------------------------------------------------------
// Sharding
// ---------------------------------------------------------------------------

/// Head/close lines replicated around a part written from an inherent impl.
type Wrapper = (String, String);

struct SPart {
    payload: String,
    wrapper: Option<Wrapper>,
}

impl SPart {
    fn file_text(&self) -> String {
        match &self.wrapper {
            None => self.payload.clone(),
            Some((head, close)) => format!("{head}{}{close}", self.payload),
        }
    }
}

enum StubLine {
    Literal(String),
    Include(usize),
}

struct Sharder<'a> {
    name: &'a str, // file stem, e.g. "block"
    lines: Vec<&'a str>,
    states: Vec<(i64, bool)>,
    parts: Vec<SPart>,
    stub: Vec<(String, StubLine)>, // (text incl. newline, kind)
}

impl<'a> Sharder<'a> {
    fn include_ref(&mut self, idx: usize, indent: &str) {
        let text = format!(
            "{indent}include!(\"{}_parts/part_{idx:03}.rs\");\n",
            self.name
        );
        self.stub.push((text, StubLine::Include(idx)));
    }

    fn literal(&mut self, text: &str) {
        self.stub.push((text.to_string(), StubLine::Literal(text.to_string())));
    }

    // The trailing `flush!()` resets `cur_size` one last time without a later read.
    #[allow(unused_assignments)]
    fn emit_region(
        &mut self,
        start: usize,
        end: usize,
        base_depth: i64,
        indent: &str,
        wrapper: Option<&Wrapper>,
    ) {
        let units = unit_boundaries(&self.lines, &self.states, start, end, base_depth);
        let mut current: Vec<(usize, usize)> = Vec::new();
        let mut cur_size = 0usize;

        macro_rules! flush {
            () => {
                if !current.is_empty() {
                    let mut payload = String::new();
                    for &(a, b) in &current {
                        for l in &self.lines[a..b] {
                            payload.push_str(l);
                        }
                    }
                    let idx = self.parts.len();
                    self.parts.push(SPart {
                        payload,
                        wrapper: wrapper.cloned(),
                    });
                    self.include_ref(idx, indent);
                    current.clear();
                    cur_size = 0;
                }
            };
        }

        for unit in units {
            let (a, b) = unit;
            let size = b - a;
            if size > THRESHOLD && wrapper.is_none() {
                match container_kind(&self.lines, &self.states, unit, base_depth) {
                    Some((ContainerKind::Mod, head_len)) => {
                        flush!();
                        for p in a..=a + head_len {
                            let l = self.lines[p];
                            self.literal(l);
                        }
                        let deeper = format!("{indent}    ");
                        self.emit_region(a + head_len + 1, b - 1, base_depth + 1, &deeper, None);
                        let close = self.lines[b - 1];
                        self.literal(close);
                        continue;
                    }
                    Some((ContainerKind::Impl, head_len)) => {
                        flush!();
                        let head: String = self.lines[a..=a + head_len].concat();
                        let mut close = self.lines[b - 1].to_string();
                        if !close.ends_with('\n') {
                            close.push('\n');
                        }
                        let w = (head, close);
                        self.emit_region(a + head_len + 1, b - 1, base_depth + 1, indent, Some(&w));
                        continue;
                    }
                    None => {}
                }
            }
            if cur_size > 0 && cur_size + size > THRESHOLD {
                flush!();
            }
            current.push(unit);
            cur_size += size;
        }
        flush!();
    }

    fn stub_text(&self, prefix: &str) -> String {
        let mut out = String::from(prefix);
        for (text, _) in &self.stub {
            out.push_str(text);
        }
        out
    }

    /// Rebuild the original single-file content from the stub structure and
    /// part payloads (dropping include! lines, merging impl-wrapper runs).
    fn reconstruct(&self, prefix: &str) -> String {
        let mut out = String::from(prefix);
        let mut pending_impl: Option<&Wrapper> = None;
        for (_, kind) in &self.stub {
            match kind {
                StubLine::Literal(text) => {
                    if let Some((_, close)) = pending_impl.take() {
                        out.push_str(close);
                    }
                    out.push_str(text);
                }
                StubLine::Include(idx) => {
                    let part = &self.parts[*idx];
                    match &part.wrapper {
                        None => {
                            if let Some((_, close)) = pending_impl.take() {
                                out.push_str(close);
                            }
                            out.push_str(&part.payload);
                        }
                        Some(w) => {
                            if pending_impl == Some(w) {
                                out.push_str(&part.payload);
                            } else {
                                if let Some((_, close)) = pending_impl.take() {
                                    out.push_str(close);
                                }
                                out.push_str(&w.0);
                                out.push_str(&part.payload);
                                pending_impl = Some(w);
                            }
                        }
                    }
                }
            }
        }
        if let Some((_, close)) = pending_impl {
            out.push_str(close);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Writes `code` for `out_file` (e.g. "block.rs") in the sharded layout:
/// stub at `OUT_DIR/out_file`, parts in `OUT_DIR/<stem>_parts/part_NNN.rs`.
/// Panics if the stub + parts do not reconstruct `code` byte-for-byte.
pub fn write_sharded_file(code: &str, out_file: &str) {
    let name = out_file
        .strip_suffix(".rs")
        .unwrap_or_else(|| panic!("sharded file without .rs suffix: {out_file}"));

    let mut owned;
    let code = if code.ends_with('\n') {
        code
    } else {
        owned = code.to_string();
        owned.push('\n');
        &*owned
    };

    let lines: Vec<&str> = code.split_inclusive('\n').collect();
    let states = line_states(code);
    assert!(
        states.len() >= lines.len(),
        "sharding {out_file}: tokenizer line count mismatch"
    );
    let prefix_len = compute_prefix_len(&lines);
    let prefix: String = lines[..prefix_len].concat();

    let mut sharder = Sharder {
        name,
        lines,
        states,
        parts: Vec::new(),
        stub: Vec::new(),
    };
    sharder.emit_region(prefix_len, sharder.lines.len(), 0, "", None);
    assert!(!sharder.parts.is_empty(), "sharding {out_file}: no parts");

    // HARD verification: stub + parts must reproduce the input exactly.
    let recon = sharder.reconstruct(&prefix);
    assert!(
        recon == code,
        "sharding {out_file}: reconstruction is not byte-identical; refusing to write"
    );

    let parts_dir = Path::new(OUT_DIR).join(format!("{name}_parts"));
    fs::create_dir_all(&parts_dir)
        .unwrap_or_else(|_| panic!("Failed to create {}", parts_dir.display()));

    for (idx, part) in sharder.parts.iter().enumerate() {
        let path = parts_dir.join(format!("part_{idx:03}.rs"));
        write_file_if_changed(&path, &part.file_text());
    }
    // Remove stale part files from previous runs with more parts.
    if let Ok(entries) = fs::read_dir(&parts_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname = fname.to_string_lossy();
            let expected = fname
                .strip_prefix("part_")
                .and_then(|s| s.strip_suffix(".rs"))
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|i| i < sharder.parts.len() && fname == format!("part_{i:03}.rs"));
            if !expected {
                fs::remove_file(entry.path()).unwrap_or_else(|_| {
                    panic!("Failed to remove stale part {}", entry.path().display())
                });
            }
        }
    }

    let stub_path = Path::new(OUT_DIR).join(out_file);
    write_file_if_changed(&stub_path, &sharder.stub_text(&prefix));
}
