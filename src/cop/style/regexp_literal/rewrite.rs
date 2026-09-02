//! Regexp literal rewrite helpers (delimiter safety and slash escaping).

pub(super) const PREFERRED_OPEN: u8 = b'{';
pub(super) const PREFERRED_CLOSE: u8 = b'}';

pub(super) fn rewrite_regexp(bytes: &[u8], to_percent: bool) -> Option<String> {
    let (body, flags) = split_body_flags(bytes)?;
    let out = if to_percent {
        if !delimiter_pair_ok(&body, PREFERRED_OPEN, PREFERRED_CLOSE) {
            return None;
        }
        build_percent(&unescape_slashes_for_percent(&body), &flags)
    } else {
        build_slash(&escape_unescaped_slashes(&body), &flags)
    };
    String::from_utf8(out).ok()
}

pub(super) fn split_body_flags(bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if bytes.starts_with(b"%r") {
        let rest = bytes.strip_prefix(b"%r")?;
        let body = pct_r_body(rest);
        Some((body.to_vec(), rest[body.len() + 2..].to_vec()))
    } else if bytes.starts_with(b"/") {
        let body = slash_regex_body(bytes);
        Some((body.to_vec(), bytes[body.len() + 2..].to_vec()))
    } else {
        None
    }
}

pub(super) fn delimiter_pair_ok(body: &[u8], open: u8, close: u8) -> bool {
    let mut depth = 0i32;
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'\\' && i + 1 < body.len() {
            i += 2;
            continue;
        }
        match body[i] {
            b if b == open => depth += 1,
            b if b == close => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
        i += 1;
    }
    depth == 0
}

pub(super) fn regex_body(bytes: &[u8]) -> &[u8] {
    if let Some(rest) = bytes.strip_prefix(b"%r") {
        pct_r_body(rest)
    } else {
        slash_regex_body(bytes)
    }
}

fn build_percent(body: &[u8], flags: &[u8]) -> Vec<u8> {
    let mut out = b"%r{".to_vec();
    out.extend_from_slice(body);
    out.push(b'}');
    out.extend_from_slice(flags);
    out
}

fn build_slash(body: &[u8], flags: &[u8]) -> Vec<u8> {
    let mut out = vec![b'/'];
    out.extend_from_slice(body);
    out.push(b'/');
    out.extend_from_slice(flags);
    out
}

fn unescape_slashes_for_percent(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'\\' && body.get(i + 1) == Some(&b'/') {
            out.push(b'/');
            i += 2;
        } else {
            out.push(body[i]);
            i += 1;
        }
    }
    out
}

fn escape_unescaped_slashes(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut i = 0;
    while i < body.len() {
        if body[i] == b'/' && !is_escaped(body, i) {
            out.extend_from_slice(b"\\/");
            i += 1;
        } else {
            out.push(body[i]);
            i += 1;
        }
    }
    out
}

fn is_escaped(bytes: &[u8], pos: usize) -> bool {
    let mut backslashes = 0usize;
    let mut j = pos;
    while j > 0 && bytes[j - 1] == b'\\' {
        backslashes += 1;
        j -= 1;
    }
    backslashes % 2 == 1
}

fn pct_r_body(rest: &[u8]) -> &[u8] {
    if rest.is_empty() {
        return rest;
    }
    let open = rest[0];
    let close = match open {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        c => c,
    };
    let inner = &rest[1..];
    if let Some(end) = inner.iter().rposition(|&b| b == close) {
        &inner[..end]
    } else {
        inner
    }
}

fn slash_regex_body(bytes: &[u8]) -> &[u8] {
    if !bytes.starts_with(b"/") || bytes.len() < 2 {
        return b"";
    }
    let mut end = bytes.len() - 1;
    while end > 1 && bytes[end] != b'/' {
        end -= 1;
    }
    if end > 1 {
        &bytes[1..end]
    } else {
        b""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_unescapes_slashes_for_percent() {
        assert_eq!(
            rewrite_regexp(br#"/\/foo/"#, true).as_deref(),
            Some("%r{/foo}")
        );
    }

    #[test]
    fn rewrite_escapes_only_unescaped_slashes() {
        assert_eq!(
            rewrite_regexp(br#"%r{\/}"#, false).as_deref(),
            Some("/\\//")
        );
    }

    #[test]
    fn rewrite_rejects_unbalanced_preferred_delimiters() {
        assert!(rewrite_regexp(br#"/}/"#, true).is_none());
    }
}
