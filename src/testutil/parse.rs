//! Parse nitrocop-style fixture annotations into source + expected offenses.

/// Expected offense from a fixture annotation.
#[derive(Debug, Clone)]
pub struct ExpectedOffense {
    pub line: usize,
    pub column: usize,
    pub cop_name: String,
    pub message: String,
}

/// Parsed fixture: clean source + expected offenses.
pub struct ParsedFixture {
    pub source: Vec<u8>,
    pub expected: Vec<ExpectedOffense>,
    pub filename: Option<String>,
}

struct RawAnnotation {
    column: usize,
    cop_name: String,
    message: String,
}

fn split_cop_message(rest: &str) -> Option<(&str, &str)> {
    let colon = rest.find(": ")?;
    let cop_name = &rest[..colon];
    if !cop_name.contains('/') {
        return None;
    }
    Some((cop_name, &rest[colon + 2..]))
}

/// Annotation: leading spaces, `^`+, space, `Dept/Cop: message`.
fn try_parse_annotation(line: &str) -> Option<RawAnnotation> {
    let trimmed = line.trim_start();
    let carets = trimmed.bytes().take_while(|&b| b == b'^').count();
    if carets == 0 || !trimmed[carets..].starts_with(' ') {
        return None;
    }
    let (cop_name, message) = split_cop_message(trimmed[carets + 1..].trim_end())?;
    Some(RawAnnotation {
        column: line.len() - trimmed.len(),
        cop_name: cop_name.to_string(),
        message: message.to_string(),
    })
}

fn try_parse_filename_directive(line: &str) -> Option<String> {
    line.strip_prefix("# rrubocop-filename: ")
        .or_else(|| line.strip_prefix("# nitrocop-filename: "))
        .map(|s| s.trim_end().to_string())
}

fn parse_line_col(loc: &str) -> Option<(usize, usize)> {
    let colon = loc.find(':')?;
    Some((loc[..colon].parse().ok()?, loc[colon + 1..].parse().ok()?))
}

fn try_parse_expect_annotation(line: &str) -> Option<ExpectedOffense> {
    let rest = line
        .strip_prefix("# rrubocop-expect: ")
        .or_else(|| line.strip_prefix("# nitrocop-expect: "))?;
    let space = rest.find(' ')?;
    let (line_num, column) = parse_line_col(&rest[..space])?;
    let (cop_name, message) = split_cop_message(rest[space + 1..].trim_end())?;
    Some(ExpectedOffense {
        line: line_num,
        column,
        cop_name: cop_name.to_string(),
        message: message.to_string(),
    })
}

fn push_caret_offense(
    source_len: usize,
    raw_idx: usize,
    element: &str,
    expected: &mut Vec<ExpectedOffense>,
) -> bool {
    let Some(annotation) = try_parse_annotation(element) else {
        return false;
    };
    assert!(
        source_len > 0,
        "Annotation on raw line {} before any source line: {element:?}",
        raw_idx + 1
    );
    expected.push(ExpectedOffense {
        line: source_len,
        column: annotation.column,
        cop_name: annotation.cop_name,
        message: annotation.message,
    });
    true
}

/// Strip annotation lines; keep Ruby source. See nitrocop `parse_fixture`.
pub fn parse_fixture(raw: &[u8]) -> ParsedFixture {
    let text = std::str::from_utf8(raw).expect("fixture must be valid UTF-8");
    let elements: Vec<&str> = text.split('\n').collect();
    let (filename, start_idx) = filename_header(&elements);
    let mut source_lines: Vec<&str> = Vec::new();
    let mut expected = Vec::new();
    for (raw_idx, element) in elements.iter().enumerate().skip(start_idx) {
        consume_fixture_line(&mut source_lines, &mut expected, raw_idx, element);
    }
    ParsedFixture {
        source: source_lines.join("\n").into_bytes(),
        expected,
        filename,
    }
}

fn filename_header(elements: &[&str]) -> (Option<String>, usize) {
    match elements.first().and_then(|l| try_parse_filename_directive(l)) {
        Some(name) => (Some(name), 1),
        None => (None, 0),
    }
}

fn consume_fixture_line<'a>(
    source_lines: &mut Vec<&'a str>,
    expected: &mut Vec<ExpectedOffense>,
    raw_idx: usize,
    element: &'a str,
) {
    if let Some(expect) = try_parse_expect_annotation(element) {
        expected.push(expect);
        return;
    }
    if push_caret_offense(source_lines.len(), raw_idx, element, expected) {
        return;
    }
    source_lines.push(element);
}
