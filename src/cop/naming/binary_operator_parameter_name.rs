//! Naming/BinaryOperatorParameterName — prefer `other` for binary ops.

use tree_sitter::Node;

use crate::cop::shared::{for_each_descendant, node_bytes, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BinaryOperatorParameterName;

const OPS: &[&[u8]] = &[
    b"|", b"^", b"&", b"<=>", b"==", b"===", b"=~", b">", b">=", b"<", b"<=", b"<<", b">>", b"+",
    b"-", b"*", b"/", b"%", b"**", b"~",
];

impl Cop for BinaryOperatorParameterName {
    fn name(&self) -> &'static str {
        "Naming/BinaryOperatorParameterName"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(param) = bad_op_param(source, node) else {
            return;
        };
        let (line, column) = source.offset_to_line_col(param.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "When defining binary operators, name the argument `other`.".into(),
        );
        if corrections.is_some() {
            let old = node_bytes(source, param).to_vec();
            push_replace(
                &mut corrections,
                param.start_byte(),
                param.end_byte(),
                "other",
                self.name(),
            );
            rename_usages(source, node, &old, &mut corrections, self.name());
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

fn bad_op_param<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    let name_node = node.child_by_field_name("name")?;
    if !OPS.contains(&node_bytes(source, name_node)) {
        return None;
    }
    let param = single_ident_param(node)?;
    let bytes = node_bytes(source, param);
    if bytes == b"other" || bytes == b"_" || bytes == b"_other" {
        None
    } else {
        Some(param)
    }
}

fn single_ident_param(node: Node<'_>) -> Option<Node<'_>> {
    let params = node.child_by_field_name("parameters")?;
    let mut cur = params.walk();
    let mut only = None;
    for n in params.named_children(&mut cur).filter(|n| n.kind() == "identifier") {
        if only.is_some() {
            return None;
        }
        only = Some(n);
    }
    only
}

fn rename_usages(
    source: &SourceFile,
    method: Node<'_>,
    old: &[u8],
    corrections: &mut Option<&mut Vec<Correction>>,
    cop_name: &'static str,
) {
    for_each_descendant(method, |n| {
        if let Some(target) = rename_target(source, n, old) {
            push_replace(
                corrections,
                target.start_byte(),
                target.end_byte(),
                "other",
                cop_name,
            );
        }
    });
}

fn rename_target<'a>(source: &SourceFile, n: Node<'a>, old: &[u8]) -> Option<Node<'a>> {
    if n.kind() == "assignment" {
        return assignment_left_ident(source, n, old);
    }
    if n.kind() != "identifier" || node_bytes(source, n) != old {
        return None;
    }
    let in_params = n
        .parent()
        .is_some_and(|p| matches!(p.kind(), "method_parameters" | "parameters"));
    (!in_params).then_some(n)
}

fn assignment_left_ident<'a>(source: &SourceFile, n: Node<'a>, old: &[u8]) -> Option<Node<'a>> {
    let left = n.child_by_field_name("left")?;
    (left.kind() == "identifier" && node_bytes(source, left) == old).then_some(left)
}
