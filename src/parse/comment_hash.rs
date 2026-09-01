//! Scan for `#` that starts a Ruby comment (not inside quotes).

pub fn first_comment_hash(bytes: &[u8]) -> Option<usize> {
    let mut i = 0;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            i = step_in_quote(bytes, i, b, q, &mut quote);
            continue;
        }
        match b {
            b'\'' | b'"' => {
                quote = Some(b);
                i += 1;
            }
            b'#' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

fn step_in_quote(bytes: &[u8], i: usize, b: u8, q: u8, quote: &mut Option<u8>) -> usize {
    if b == b'\\' && q == b'"' && i + 1 < bytes.len() {
        return i + 2;
    }
    if b == q {
        *quote = None;
    }
    i + 1
}
