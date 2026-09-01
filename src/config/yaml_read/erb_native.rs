//! Native simple-ERB expansion (literals / vars / ENV).

use std::collections::HashMap;

use super::erb_expr::{eval_expr, is_ident};

enum Tag {
    Stmt,
    Expr,
    Comment,
}

/// Expand simple ERB without Ruby. Returns `None` for anything unsupported.
pub(super) fn expand_erb_native(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut rest = raw;
    while let Some(open) = rest.find("<%") {
        out.push_str(&rest[..open]);
        rest = apply_next_tag(&mut out, &mut vars, &rest[open + 2..])?;
    }
    out.push_str(rest);
    Some(out)
}

fn apply_next_tag<'a>(
    out: &mut String,
    vars: &mut HashMap<String, String>,
    after: &'a str,
) -> Option<&'a str> {
    if after.starts_with('%') {
        return None;
    }
    let (kind, body) = tag_body(after)?;
    let close = body.find("%>")?;
    apply_tag(kind, &body[..close], out, vars)?;
    Some(&body[close + 2..])
}

fn tag_body(after: &str) -> Option<(Tag, &str)> {
    match after.bytes().next()? {
        b'=' => Some((Tag::Expr, &after[1..])),
        b'#' => Some((Tag::Comment, &after[1..])),
        _ => Some((Tag::Stmt, after)),
    }
}

fn apply_tag(
    kind: Tag,
    inner: &str,
    out: &mut String,
    vars: &mut HashMap<String, String>,
) -> Option<()> {
    match kind {
        Tag::Comment => Some(()),
        Tag::Expr => {
            out.push_str(&eval_expr(inner, vars)?);
            Some(())
        }
        Tag::Stmt => apply_stmt(inner, vars),
    }
}

fn apply_stmt(stmt: &str, vars: &mut HashMap<String, String>) -> Option<()> {
    for part in stmt.split([';', '\n']) {
        apply_one_stmt(part.trim(), vars)?;
    }
    Some(())
}

fn apply_one_stmt(part: &str, vars: &mut HashMap<String, String>) -> Option<()> {
    if part.is_empty() {
        return Some(());
    }
    let eq = part.find('=')?;
    let name = part[..eq].trim();
    let rhs = part[eq + 1..].trim();
    if rhs.starts_with('=') || !is_ident(name) {
        return None;
    }
    let value = eval_expr(rhs, vars)?;
    vars.insert(name.to_string(), value);
    Some(())
}
