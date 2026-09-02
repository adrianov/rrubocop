//! Style/CaseLikeIf — replace case-like `if-elsif` with `case-when`.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseLikeIf;

fn unwrap(node: Node<'_>) -> Node<'_> {
    let mut n = node;
    while matches!(n.kind(), "begin" | "parenthesized_statements") {
        let mut cur = n.walk();
        if let Some(inner) = n.named_children(&mut cur).next() {
            n = inner;
        } else {
            break;
        }
    }
    n
}

fn binary_op<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<&'a [u8]> {
    if node.kind() != "binary" {
        return None;
    }
    let mut cur = node.walk();
    node.children(&mut cur)
        .find(|c| !c.is_named())
        .map(|c| node_bytes(source, c))
}

fn sides(node: Node<'_>) -> Option<(Node<'_>, Node<'_>)> {
    Some((
        node.child_by_field_name("left")?,
        node.child_by_field_name("right")?,
    ))
}

fn same_target(source: &SourceFile, a: Node<'_>, b: Node<'_>) -> bool {
    a == b
        || (a.kind() == "identifier"
            && b.kind() == "identifier"
            && node_bytes(source, a) == node_bytes(source, b))
}

fn match_target<'a>(_source: &'a SourceFile, lhs: Node<'a>, rhs: Node<'a>) -> Option<Node<'a>> {
    if lhs.kind() == "regex" {
        return (rhs.kind() == "identifier").then_some(rhs);
    }
    if rhs.kind() == "regex" {
        return (lhs.kind() == "identifier").then_some(lhs);
    }
    None
}

fn find_target<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let n = unwrap(node);
    if let Some(op) = binary_op(source, n) {
        let (l, r) = sides(n)?;
        return match op {
            b"=~" | b"==" | b"eql?" | b"equal?" | b"===" => {
                if op == b"=~" {
                    match_target(source, l, r)
                } else {
                    equality_target(source, l, r)
                }
            }
            _ => None,
        };
    }
    if n.kind() == "call" {
        let meth = call_method_name(source, n)?;
        let recv = call_receiver(n)?;
        let args = crate::cop::shared::argument_nodes(n);
        let arg = args.first().copied()?;
        return match meth {
            b"=~" | b"match" | b"match?" => match_target(source, recv, arg),
            b"is_a?" if recv.kind() == "identifier" => Some(arg),
            b"include?" | b"cover?" if recv.kind() == "range" => Some(recv),
            _ => None,
        };
    }
    if n.kind() == "binary" && binary_op(source, n) == Some(b"||") {
        return find_target(source, sides(n)?.0);
    }
    None
}

fn literal_or_const(source: &SourceFile, n: Node<'_>) -> bool {
    matches!(
        n.kind(),
        "string" | "symbol" | "simple_symbol" | "integer" | "float" | "true" | "false" | "nil"
    ) || (n.kind() == "constant" && node_bytes(source, n).iter().all(|&b| b.is_ascii_uppercase()))
}

fn equality_target<'a>(source: &'a SourceFile, l: Node<'a>, r: Node<'a>) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if literal_or_const(source, l) && r.kind() == "identifier" {
        return Some(r);
    }
    if literal_or_const(source, r) && l.kind() == "identifier" {
        return Some(l);
    }
    None
}

fn collect_condition(source: &SourceFile, node: Node<'_>, target: Node<'_>) -> bool {
    let n = unwrap(node);
    if n.kind() == "binary" && binary_op(source, n) == Some(b"||") {
        let (l, r) = sides(n).unwrap();
        return collect_condition(source, l, target) && collect_condition(source, r, target);
    }
    if let Some(op) = binary_op(source, n) {
        let (l, r) = sides(n).unwrap();
        return match op {
            b"=~" => match_side(source, l, r, target).is_some(),
            b"==" | b"eql?" | b"equal?" | b"===" => equality_side(source, l, r, target).is_some(),
            _ => false,
        };
    }
    if n.kind() == "call" {
        let meth = call_method_name(source, n).unwrap_or(b"");
        let recv = call_receiver(n);
        let arg = crate::cop::shared::argument_nodes(n).first().copied();
        return match meth {
            b"=~" | b"match" | b"match?" => match (recv, arg) {
                (Some(r), Some(a)) => match_side(source, r, a, target).is_some(),
                _ => false,
            },
            b"is_a?" => recv.is_some_and(|r| same_target(source, r, target)),
            _ => false,
        };
    }
    false
}

fn match_side<'a>(
    source: &'a SourceFile,
    l: Node<'a>,
    r: Node<'a>,
    target: Node<'a>,
) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if same_target(source, l, target) && r.kind() == "regex" {
        return Some(r);
    }
    if same_target(source, r, target) && l.kind() == "regex" {
        return Some(l);
    }
    None
}

fn equality_side<'a>(
    source: &'a SourceFile,
    l: Node<'a>,
    r: Node<'a>,
    target: Node<'a>,
) -> Option<Node<'a>> {
    let l = unwrap(l);
    let r = unwrap(r);
    if same_target(source, l, target) && literal_or_const(source, r) {
        return Some(r);
    }
    if same_target(source, r, target) && literal_or_const(source, l) {
        return Some(l);
    }
    None
}

fn if_condition(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(c) = node.child_by_field_name("condition") {
        return Some(c);
    }
    let mut cur = node.walk();
    for child in node.named_children(&mut cur) {
        match child.kind() {
            "then" | "else" | "elsif" => break,
            "if" | "unless" | "comment" => continue,
            _ => return Some(child),
        }
    }
    None
}

fn if_alternative(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(alt) = node.child_by_field_name("alternative") {
        return Some(alt);
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|c| matches!(c.kind(), "elsif" | "else"))
}

fn branch_conditions(mut node: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    loop {
        if let Some(cond) = if_condition(node) {
            out.push(cond);
        }
        let Some(alt) = if_alternative(node) else {
            break;
        };
        if alt.kind() != "elsif" {
            break;
        }
        node = alt;
    }
    out
}

fn should_check(node: Node<'_>) -> bool {
    node.kind() == "if"
        && node.parent().is_none_or(|p| p.kind() != "elsif")
        && if_alternative(node).is_some_and(|a| a.kind() == "elsif")
}

impl Cop for CaseLikeIf {
    fn name(&self) -> &'static str {
        "Style/CaseLikeIf"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !should_check(node) {
            return;
        }
        let conditions = branch_conditions(node);
        if conditions.len() < 2 {
            return;
        }
        let Some(target) = find_target(source, conditions[0]) else {
            return;
        };
        if !conditions
            .iter()
            .all(|c| collect_condition(source, *c, target))
        {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Convert `if-elsif` to `case-when`.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CaseLikeIf, "cops/style/case_like_if");
}
