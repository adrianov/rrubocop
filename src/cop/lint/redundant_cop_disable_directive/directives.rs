use std::collections::HashMap;

use crate::parse::source::SourceFile;

#[derive(Clone, Debug)]
pub(super) struct DisableDirective {
    pub line: usize,
    pub column: usize,
    pub cops: Vec<String>,
    pub range: (usize, usize),
}

pub(super) fn disable_marker(line: &str) -> Option<(usize, &str)> {
    let lower = line.to_ascii_lowercase();
    let marker = "# rubocop:disable";
    let pos = lower.find(marker)?;
    Some((pos, line[pos + marker.len()..].trim()))
}

fn enable_marker(line: &str) -> Option<&str> {
    let lower = line.to_ascii_lowercase();
    let marker = "# rubocop:enable";
    let pos = lower.find(marker)?;
    Some(line[pos + marker.len()..].trim())
}

pub(super) fn cop_names(rest: &str) -> Vec<String> {
    let rest = rest.trim_start_matches(':').trim();
    let cops = rest
        .split("--")
        .next()
        .unwrap_or("")
        .split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>();
    if cops.is_empty() {
        vec!["all".into()]
    } else {
        cops
    }
}

fn open_block_key(open: &HashMap<String, DisableDirective>, name: String) -> String {
    open.keys()
        .find(|k| k.eq_ignore_ascii_case(&name))
        .cloned()
        .unwrap_or(name)
}

fn apply_enable(
    open: &mut HashMap<String, DisableDirective>,
    out: &mut Vec<DisableDirective>,
    line_no: usize,
    rest: &str,
) {
    let names = cop_names(rest);
    if names.iter().any(|n| n.eq_ignore_ascii_case("all")) {
        for (_, mut dir) in open.drain() {
            dir.range = (dir.range.0, line_no);
            out.push(dir);
        }
        return;
    }
    for name in names {
        let key = open_block_key(open, name);
        if let Some(mut dir) = open.remove(&key) {
            dir.range = (dir.range.0, line_no);
            out.push(dir);
        }
    }
}

fn record_disable(
    open: &mut HashMap<String, DisableDirective>,
    out: &mut Vec<DisableDirective>,
    line_no: usize,
    col: usize,
    line: &str,
    cops: Vec<String>,
) {
    if line.trim_start().starts_with('#') {
        for name in cops {
            open.insert(
                name.clone(),
                DisableDirective {
                    line: line_no,
                    column: col,
                    cops: vec![name],
                    range: (line_no, line_no),
                },
            );
        }
    } else {
        out.push(DisableDirective {
            line: line_no,
            column: col,
            cops,
            range: (line_no, line_no),
        });
    }
}

fn flush_open_directives(
    open: HashMap<String, DisableDirective>,
    end_line: usize,
    out: &mut Vec<DisableDirective>,
) {
    for (_, mut dir) in open {
        dir.range = (dir.range.0, end_line);
        out.push(dir);
    }
}

pub(super) fn collect_directives(source: &SourceFile) -> Vec<DisableDirective> {
    let mut out = Vec::new();
    let mut open = HashMap::new();
    for (i, line) in source.lines().enumerate() {
        let s = String::from_utf8_lossy(line);
        let line_no = i + 1;
        if let Some(rest) = enable_marker(&s) {
            apply_enable(&mut open, &mut out, line_no, rest);
        }
        if let Some((col, rest)) = disable_marker(&s) {
            record_disable(&mut open, &mut out, line_no, col, &s, cop_names(rest));
        }
    }
    flush_open_directives(open, source.line_count().max(1), &mut out);
    out
}
