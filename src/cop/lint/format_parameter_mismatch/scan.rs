//! Percent-format field scanner for Lint/FormatParameterMismatch.
//!
//! Arity matches RuboCop `FormatString::FormatSequence#arity`:
//! each `*` in width/precision plus one for the conversion type.

pub(super) fn field_count(fmt: &str) -> Option<(usize, bool)> {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    let mut count = 0;
    let mut named = false;
    while let Some((next, arity, is_named)) = scan_percent(bytes, i) {
        count += arity;
        named |= is_named;
        i = next;
    }
    Some((count, named))
}

fn scan_percent(bytes: &[u8], i: usize) -> Option<(usize, usize, bool)> {
    let i = find_percent(bytes, i)?;
    Some(after_percent(bytes, i))
}

fn find_percent(bytes: &[u8], mut i: usize) -> Option<usize> {
    while i < bytes.len() {
        if bytes[i] == b'%' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Returns `(next_index, arity, is_named)`.
fn after_percent(bytes: &[u8], mut i: usize) -> (usize, usize, bool) {
    i += 1;
    if let Some(early) = named_or_escaped(bytes, i) {
        return early;
    }
    let (i, stars) = skip_flags(bytes, i);
    if i >= bytes.len() {
        (i, 0, false)
    } else {
        (i + 1, stars + 1, false)
    }
}

fn named_or_escaped(bytes: &[u8], i: usize) -> Option<(usize, usize, bool)> {
    match bytes.get(i) {
        Some(&b'%') => Some((i + 1, 0, false)),
        Some(&b'{') => Some((consume_until(bytes, i, b'}').saturating_add(1), 1, true)),
        Some(&b'<') => Some((consume_angle_named(bytes, i + 1), 1, true)),
        _ => None,
    }
}

fn consume_until(bytes: &[u8], mut i: usize, stop: u8) -> usize {
    while i < bytes.len() && bytes[i] != stop {
        i += 1;
    }
    i
}

fn consume_angle_named(bytes: &[u8], mut i: usize) -> usize {
    i = consume_until(bytes, i, b'>');
    if i < bytes.len() {
        i += 1;
    }
    if i < bytes.len() {
        i += 1;
    }
    i
}

/// Skip flags / width / precision; return `(index_at_type, star_count)`.
fn skip_flags(bytes: &[u8], mut i: usize) -> (usize, usize) {
    while i < bytes.len() && b"-0+ #".contains(&bytes[i]) {
        i += 1;
    }
    // Numbered arg flags like `%2$d`.
    i = skip_numbered(bytes, i);
    let mut stars = 0;
    // width
    let (ni, w) = skip_number(bytes, i);
    i = ni;
    stars += w;
    // precision
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let (ni, p) = skip_number(bytes, i);
        i = ni;
        stars += p;
    }
    (i, stars)
}

fn skip_numbered(bytes: &[u8], mut i: usize) -> usize {
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > start && bytes.get(i) == Some(&b'$') {
        i + 1
    } else {
        start
    }
}

/// Digits, or `*` with optional `\d+$` (dynamic width/precision).
fn skip_number(bytes: &[u8], mut i: usize) -> (usize, usize) {
    if bytes.get(i) == Some(&b'*') {
        i += 1;
        i = skip_numbered(bytes, i);
        return (i, 1);
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    (i, 0)
}
