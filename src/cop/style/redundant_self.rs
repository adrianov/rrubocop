//! Style/RedundantSelf — avoid unnecessary self.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantSelf;

impl Cop for RedundantSelf {
    fn name(&self) -> &'static str {
        "Style/RedundantSelf"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(recv) = call_receiver(node) else {
            return;
        };
        if recv.kind() != "self" || !self_is_redundant(source, node) {
            return;
        }
        report(self, source, node, recv, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &RedundantSelf,
    source: &SourceFile,
    node: Node<'_>,
    recv: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(recv.start_byte());
    let mut diag = cop.diagnostic(source, line, col, "Redundant `self` detected.".to_string());
    if let Some(corr) = corrections.as_mut() {
        let meth = node.child_by_field_name("method").unwrap_or(node);
        corr.push(Correction {
            start: recv.start_byte(),
            end: meth.start_byte(),
            replacement: String::new(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn self_is_redundant(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    if method == b"class" || method.starts_with(b"[") {
        return false;
    }
    if !method
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'!' | b'?' | b'='))
    {
        return false;
    }
    // `self.foo` is required when a local/parameter/`foo=` shadows `foo`.
    !name_is_shadowed(source, node, method)
}

fn name_is_shadowed(source: &SourceFile, node: Node<'_>, method: &[u8]) -> bool {
    let bare = method.strip_suffix(b"=").unwrap_or(method);
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "method" | "singleton_method" | "block" | "do_block" | "lambda") {
            if params_include(source, n, bare) {
                return true;
            }
            if lvars_before(source, n, node, bare) {
                return true;
            }
        }
        if matches!(n.kind(), "class" | "module" | "program") {
            break;
        }
        p = n.parent();
    }
    false
}

fn params_include(source: &SourceFile, scope: Node<'_>, name: &[u8]) -> bool {
    let mut cur = scope.walk();
    for child in scope.named_children(&mut cur) {
        if matches!(
            child.kind(),
            "method_parameters"
                | "parameters"
                | "block_parameters"
                | "lambda_parameters"
                | "identifier"
                | "optional_parameter"
                | "keyword_parameter"
                | "splat_parameter"
                | "hash_splat_parameter"
                | "block_parameter"
        ) {
            if param_name_match(source, child, name) {
                return true;
            }
            let mut c2 = child.walk();
            for grand in child.named_children(&mut c2) {
                if param_name_match(source, grand, name) {
                    return true;
                }
            }
        }
    }
    false
}

fn param_name_match(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    match node.kind() {
        "identifier" => node_bytes(source, node) == name,
        "optional_parameter" | "keyword_parameter" | "splat_parameter"
        | "hash_splat_parameter" | "block_parameter" => node
            .child_by_field_name("name")
            .or_else(|| {
                let mut cur = node.walk();
                node.named_children(&mut cur).find(|n| n.kind() == "identifier")
            })
            .is_some_and(|n| node_bytes(source, n) == name),
        _ => false,
    }
}

fn lvars_before(source: &SourceFile, scope: Node<'_>, before: Node<'_>, name: &[u8]) -> bool {
    let mut found = false;
    shared_for_each_before(scope, before, &mut |n| {
        if n.kind() == "assignment" {
            if let Some(left) = n.child_by_field_name("left") {
                if left.kind() == "identifier" && node_bytes(source, left) == name {
                    found = true;
                }
            }
        }
    });
    found
}

fn shared_for_each_before(root: Node<'_>, before: Node<'_>, f: &mut impl FnMut(Node<'_>)) {
    fn walk(node: Node<'_>, before: Node<'_>, f: &mut impl FnMut(Node<'_>)) -> bool {
        if node.id() == before.id() {
            return true;
        }
        f(node);
        let mut cur = node.walk();
        for child in node.children(&mut cur) {
            if walk(child, before, f) {
                return true;
            }
        }
        false
    }
    walk(root, before, f);
}
