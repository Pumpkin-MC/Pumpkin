use crate::tag::NbtTag;

/// Evaluate an NBT path against a root tag and return all selected tags.
///
/// This implements the core Java Edition NBT path semantics described on the wiki:
/// - Dot-separated nodes, with dots allowed to be omitted before [] nodes.
/// - Named child selection, including quoted names ('..' or "..") for special characters.
/// - Optional empty compound filters: name{} (requires the named child to be a compound),
///   and list filters: [{}} selects only compound elements. Non-empty filter bodies are parsed
///   but not matched yet (treated like empty filters).
/// - List/array element selection: [index] (supports negative indices) and [] (all elements).
/// - Root filter {} as the first node selects the root compound (non-empty content is accepted but not matched yet).
pub fn get_tag_by_path(root: &NbtTag, path: &str) -> Vec<NbtTag> {
    // Empty path selects the root
    if path.is_empty() {
        return vec![root.clone()];
    }

    let mut collection = vec![root.clone()];
    for seg in split_path(path) {
        collection = apply_segment(collection, &seg);
        if collection.is_empty() {
            break;
        }
    }
    collection
}

/// Splits by '.' but respects quotes (single/double) and nesting of [] and {}
fn split_path(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut quote: Option<char> = None;
    let mut escape = false;
    for ch in path.chars() {
        if let Some(q) = quote {
            cur.push(ch);
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                quote = Some(ch);
                cur.push(ch);
            }
            '.' if bracket == 0 && brace == 0 => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
            }
            '[' => {
                bracket += 1;
                cur.push(ch);
            }
            ']' => {
                bracket = bracket.saturating_sub(1);
                cur.push(ch);
            }
            '{' => {
                brace += 1;
                cur.push(ch);
            }
            '}' => {
                brace = brace.saturating_sub(1);
                cur.push(ch);
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote_name(s: &str) -> Option<String> {
    let mut chars = s.chars();
    let first = chars.next()?;
    let last = s.chars().last()?;
    if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
        let inner = &s[1..s.len() - 1];
        let mut out = String::new();
        let mut esc = false;
        for ch in inner.chars() {
            if esc {
                out.push(ch);
                esc = false;
                continue;
            }
            if ch == '\\' {
                esc = true;
            } else {
                out.push(ch);
            }
        }
        Some(out)
    } else {
        None
    }
}

// replaced below with extended version supporting compound filter content

fn apply_segment(current: Vec<NbtTag>, segment: &str) -> Vec<NbtTag> {
    let seg = segment.trim();
    if seg.is_empty() {
        return current;
    }

    // Root filter node like {} or {foo:1b} treated as selecting the root (and require compound if braces present)
    if seg.starts_with('{') && seg.ends_with('}') {
        let inner = &seg[1..seg.len() - 1];
        let filter_str = inner.trim();
        let filter = if filter_str.is_empty() {
            None
        } else {
            Some(filter_str)
        };
        let mut out = Vec::new();
        for tag in current {
            if let NbtTag::Compound(ref c) = tag {
                if let Some(f) = filter {
                    if match_compound_filter_str(c, f) {
                        out.push(tag);
                    }
                } else {
                    out.push(tag);
                }
            }
        }
        return out;
    }

    // Parse name (possibly quoted) and optional name-level compound filter, followed by bracket ops
    let (name_opt, after_name) = if seg.starts_with('"') || seg.starts_with('\'') {
        // quoted name
        let quote = seg.chars().next().unwrap();
        let mut i = 1usize;
        let bytes: Vec<char> = seg.chars().collect();
        let mut esc = false;
        while i < bytes.len() {
            let ch = bytes[i];
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == quote {
                i += 1; // include closing quote
                break;
            }
            i += 1;
        }
        let name_raw = &seg[..i];
        let name = unquote_name(name_raw).unwrap_or_else(|| name_raw.to_string());
        (Some(name), &seg[i..])
    } else if seg.starts_with('[') {
        (None, seg)
    } else {
        // bare name until next '{' or '['
        let mut i = 0usize;
        for (idx, ch) in seg.char_indices() {
            if ch == '{' || ch == '[' {
                break;
            }
            i = idx + ch.len_utf8();
        }
        let name = &seg[..i];
        (
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
            &seg[i..],
        )
    };

    let (name_filter_content, after_filter) = if after_name.starts_with('{') {
        // scan to matching }
        let mut depth = 0usize;
        let mut end = 0usize;
        for (idx, ch) in after_name.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = idx + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        let content = &after_name[..end];
        let inner = &content[1..content.len() - 1];
        let trimmed = inner.trim();
        (Some(trimmed.to_string()), &after_name[end..])
    } else {
        (None, after_name)
    };

    let ops = parse_bracket_ops(after_filter);

    // Apply name selection first
    let mut next: Vec<NbtTag> = Vec::new();
    if let Some(name) = name_opt {
        for tag in current {
            if let NbtTag::Compound(c) = tag
                && let Some(child) = c.get(&name)
            {
                // If a compound filter is present at the name-level, require compound and optionally match
                match &name_filter_content {
                    Some(filter_str) => {
                        if let NbtTag::Compound(comp) = child
                            && (filter_str.is_empty()
                                || match_compound_filter_str(comp, filter_str))
                        {
                            next.push(child.clone());
                        }
                    }
                    None => next.push(child.clone()),
                }
            }
        }
    } else {
        next = current;
    }

    // Apply bracket ops in order
    for op in ops {
        match op {
            BracketOp::All => {
                let mut out = Vec::new();
                for tag in next {
                    match tag {
                        NbtTag::List(list) => out.extend(list.into_iter()),
                        NbtTag::ByteArray(bytes) => {
                            for b in bytes.iter() {
                                out.push(NbtTag::Byte(*b as i8));
                            }
                        }
                        NbtTag::IntArray(ints) => {
                            for i in ints {
                                out.push(NbtTag::Int(i));
                            }
                        }
                        NbtTag::LongArray(longs) => {
                            for l in longs {
                                out.push(NbtTag::Long(l));
                            }
                        }
                        _ => {}
                    }
                }
                next = out;
            }
            BracketOp::Index(idx) => {
                let mut out = Vec::new();
                for tag in next {
                    match tag {
                        NbtTag::List(list) => {
                            let len = list.len() as isize;
                            let mut i = idx;
                            if i < 0 {
                                i += len;
                            }
                            if i >= 0 && i < len {
                                out.push(list[i as usize].clone());
                            }
                        }
                        NbtTag::ByteArray(bytes) => {
                            let len = bytes.len() as isize;
                            let mut i = idx;
                            if i < 0 {
                                i += len;
                            }
                            if i >= 0 && i < len {
                                out.push(NbtTag::Byte(bytes[i as usize] as i8));
                            }
                        }
                        NbtTag::IntArray(ints) => {
                            let len = ints.len() as isize;
                            let mut i = idx;
                            if i < 0 {
                                i += len;
                            }
                            if i >= 0 && i < len {
                                out.push(NbtTag::Int(ints[i as usize]));
                            }
                        }
                        NbtTag::LongArray(longs) => {
                            let len = longs.len() as isize;
                            let mut i = idx;
                            if i < 0 {
                                i += len;
                            }
                            if i >= 0 && i < len {
                                out.push(NbtTag::Long(longs[i as usize]));
                            }
                        }
                        _ => {}
                    }
                }
                next = out;
            }
            BracketOp::CompoundFilter { content } => {
                let mut out = Vec::new();
                for tag in next {
                    if let NbtTag::List(list) = tag {
                        for el in list {
                            if let NbtTag::Compound(ref comp) = el {
                                if let Some(ref s) = content {
                                    if match_compound_filter_str(comp, s) {
                                        out.push(el);
                                    }
                                } else {
                                    out.push(el);
                                }
                            }
                        }
                    }
                }
                next = out;
            }
        }
    }

    next
}

enum BracketOp {
    All,
    Index(isize),
    /// Filter only compound elements with optional content (inside {}). None => just compounds.
    CompoundFilter {
        content: Option<String>,
    },
}

// Parse bracket operations including optional compound filter content.
fn parse_bracket_ops(mut s: &str) -> Vec<BracketOp> {
    let mut ops = Vec::new();
    while let Some(start) = s.find('[') {
        s = &s[start + 1..];
        let close = match s.find(']') {
            Some(i) => i,
            None => break,
        };
        let content = &s[..close].trim();
        if content.is_empty() {
            ops.push(BracketOp::All);
        } else if content.starts_with('{') && content.ends_with('}') {
            let inner = &content[1..content.len() - 1];
            let trimmed = inner.trim();
            let op = if trimmed.is_empty() {
                BracketOp::CompoundFilter { content: None }
            } else {
                BracketOp::CompoundFilter {
                    content: Some(trimmed.to_string()),
                }
            };
            ops.push(op);
        } else if let Ok(idx) = content.parse::<isize>() {
            ops.push(BracketOp::Index(idx));
        }
        s = &s[close + 1..];
    }
    ops
}

// Very small filter matcher for flat key:value pairs; supports strings and numeric suffixes b/s/l/f/d (case-insensitive).
fn match_compound_filter_str(comp: &crate::compound::NbtCompound, filter_str: &str) -> bool {
    if let Some(pairs) = parse_simple_filter_pairs(filter_str) {
        for (k, v) in pairs {
            let Some(actual) = comp.get(&k) else {
                return false;
            };
            if actual != &v {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn parse_simple_filter_pairs(s: &str) -> Option<Vec<(String, NbtTag)>> {
    // Split on commas at top level, respecting quotes.
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut esc = false;
    let mut depth_brace = 0usize;
    let mut depth_bracket = 0usize;
    for ch in s.chars() {
        if let Some(q) = quote {
            cur.push(ch);
            if esc {
                esc = false;
                continue;
            }
            if ch == '\\' {
                esc = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                quote = Some(ch);
                cur.push(ch);
            }
            '{' => {
                depth_brace += 1;
                cur.push(ch);
            }
            '}' => {
                depth_brace = depth_brace.saturating_sub(1);
                cur.push(ch);
            }
            '[' => {
                depth_bracket += 1;
                cur.push(ch);
            }
            ']' => {
                depth_bracket = depth_bracket.saturating_sub(1);
                cur.push(ch);
            }
            ',' if depth_brace == 0 && depth_bracket == 0 => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }

    let mut out = Vec::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let (k, v) = split_key_value(&part)?;
        let key = parse_key_name(k)?;
        let value = parse_simple_value(v.trim())?;
        out.push((key, value));
    }
    Some(out)
}

fn split_key_value(s: &str) -> Option<(&str, &str)> {
    // split at first ':' outside quotes/braces
    let mut quote: Option<char> = None;
    let mut depth_brace = 0usize;
    for (i, ch) in s.char_indices() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else if ch == '\\' { /* skip next in full impl */
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '{' => depth_brace += 1,
            '}' => {
                depth_brace = depth_brace.saturating_sub(1);
            }
            ':' if depth_brace == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

fn parse_key_name(s: &str) -> Option<String> {
    let st = s.trim();
    if let Some(q) = st.chars().next()
        && (q == '"' || q == '\'')
    {
        return unquote_name(st);
    }
    Some(st.to_string())
}

fn parse_simple_value(s: &str) -> Option<NbtTag> {
    let st = s.trim();
    if st.is_empty() {
        return None;
    }
    // Quoted string
    if let Some(q) = st.chars().next()
        && (q == '"' || q == '\'')
    {
        return unquote_name(st).map(NbtTag::String);
    }
    // Numeric with suffix
    let lower = st.to_ascii_lowercase();
    if let Some(num) = lower.strip_suffix('b') {
        return num.trim().parse::<i8>().ok().map(NbtTag::Byte);
    }
    if let Some(num) = lower.strip_suffix('s') {
        return num.trim().parse::<i16>().ok().map(NbtTag::Short);
    }
    if let Some(num) = lower.strip_suffix('l') {
        return num.trim().parse::<i64>().ok().map(NbtTag::Long);
    }
    if let Some(num) = lower.strip_suffix('f') {
        return num.trim().parse::<f32>().ok().map(NbtTag::Float);
    }
    if let Some(num) = lower.strip_suffix('d') {
        return num.trim().parse::<f64>().ok().map(NbtTag::Double);
    }
    // Plain integer
    if let Ok(i) = st.parse::<i32>() {
        return Some(NbtTag::Int(i));
    }
    // Plain float
    if let Ok(f) = st.parse::<f64>() {
        return Some(NbtTag::Double(f));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compound::NbtCompound;

    fn sample_root() -> NbtTag {
        let mut root = NbtCompound::new();
        root.put("foo", NbtTag::Int(42));
        root.put(
            "Pos",
            NbtTag::List(vec![
                NbtTag::Double(1.0),
                NbtTag::Double(2.0),
                NbtTag::Double(3.0),
            ]),
        );
        let mut cmp = NbtCompound::new();
        cmp.put("A [crazy name]!", NbtTag::Compound(NbtCompound::new()));
        root.put("bar", NbtTag::Compound(cmp));
        let list = vec![
            NbtTag::Compound(NbtCompound::new()),
            NbtTag::Int(5),
            NbtTag::Compound(NbtCompound::new()),
        ];
        root.put("Inventory", NbtTag::List(list));
        NbtTag::Compound(root)
    }

    #[test]
    fn test_named() {
        let root = sample_root();
        let out = get_tag_by_path(&root, "foo");
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0], NbtTag::Int(42)));
    }

    #[test]
    fn test_index_and_negative() {
        let root = sample_root();
        let out0 = get_tag_by_path(&root, "Pos[0]");
        assert_eq!(out0.len(), 1);
        assert!(matches!(out0[0], NbtTag::Double(1.0)));
        let out_last = get_tag_by_path(&root, "Pos[-1]");
        assert_eq!(out_last.len(), 1);
        assert!(matches!(out_last[0], NbtTag::Double(3.0)));
    }

    #[test]
    fn test_all_elements_and_compound_filter() {
        let root = sample_root();
        let all = get_tag_by_path(&root, "Inventory[]");
        assert_eq!(all.len(), 3);
        let only_cmp = get_tag_by_path(&root, "Inventory[{}]");
        assert_eq!(only_cmp.len(), 2);
    }

    #[test]
    fn test_quoted_name() {
        let root = sample_root();
        let out = get_tag_by_path(&root, "bar.\"A [crazy name]!\"");
        assert_eq!(out.len(), 1);
        assert!(matches!(out[0], NbtTag::Compound(_)));
    }
}
