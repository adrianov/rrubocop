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
    if b == b'\\' && i + 1 < bytes.len() {
        let next = bytes[i + 1];
        // Double quotes: any `\X`. Single quotes: only `\'` and `\\`.
        if q == b'"' || matches!(next, b'\\' | b'\'') {
            return i + 2;
        }
    }
    if b == q {
        *quote = None;
    }
    i + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_after_escaped_single_quote() {
        let line = "it 'doesn\\'t cancel' do # rubocop:disable RSpec/MultipleExpectations";
        let idx = first_comment_hash(line.as_bytes()).unwrap();
        assert_eq!(&line[idx..], "# rubocop:disable RSpec/MultipleExpectations");
    }

    #[test]
    fn hash_inside_single_quoted_string() {
        assert_eq!(first_comment_hash(b"'a # b'"), None);
    }
}
