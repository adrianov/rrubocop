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
        if recv.kind() != "self" {
            return;
        }
        // `self.foo = bar` is often `assignment` with call LHS, not `foo=`.
        if call_is_assign_lhs(node) {
            return;
        }
        // `self.foo[i] =` / `self.foo[i] ||=` — self required for attr + index write.
        if call_is_index_assign_recv(node) {
            return;
        }
        if !self_is_redundant(source, node) {
            return;
        }
        // RuboCop tracks assignment LHS names in a shared class-level scope; once
        // `self.data ||= {}` appears, `self.data` reads elsewhere aren't flagged.
        if let Some(method) = call_method_name(source, node) {
            let bare = method.strip_suffix(b"=").unwrap_or(method);
            if self_assign_names_in_enclosing_type(source, node)
                .iter()
                .any(|n| n.as_slice() == bare)
            {
                return;
            }
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

fn call_is_assign_lhs(node: Node<'_>) -> bool {
    node.parent().is_some_and(|p| {
        matches!(p.kind(), "assignment" | "operator_assignment")
            && p.child_by_field_name("left").is_some_and(|l| l.id() == node.id())
    })
}

/// `self.rates[id] ||= {}` — walk through element_reference to assignment.
fn call_is_index_assign_recv(node: Node<'_>) -> bool {
    let mut cur = node;
    for _ in 0..4 {
        let Some(parent) = cur.parent() else {
            return false;
        };
        if matches!(parent.kind(), "assignment" | "operator_assignment") {
            return parent
                .child_by_field_name("left")
                .is_some_and(|l| l.id() == cur.id() || contains_node(l, node));
        }
        if matches!(parent.kind(), "element_reference" | "call") {
            // Continue if we're the receiver of [] / []=
            let recv = parent
                .child_by_field_name("object")
                .or_else(|| parent.child_by_field_name("receiver"));
            if recv.is_some_and(|r| r.id() == cur.id()) {
                cur = parent;
                continue;
            }
        }
        return false;
    }
    false
}

fn contains_node(root: Node<'_>, target: Node<'_>) -> bool {
    if root.id() == target.id() {
        return true;
    }
    let mut cur = root.walk();
    root.named_children(&mut cur).any(|c| contains_node(c, target))
}

/// Names assigned via `self.foo` / `self.foo[…] =` under the same class/module
/// (RuboCop RedundantSelf class-scope local-name leakage parity).
fn self_assign_names_in_enclosing_type(source: &SourceFile, node: Node<'_>) -> Vec<Vec<u8>> {
    let mut p = node.parent();
    let mut type_node = None;
    while let Some(n) = p {
        if matches!(n.kind(), "class" | "module" | "singleton_class") {
            type_node = Some(n);
            break;
        }
        p = n.parent();
    }
    let Some(root) = type_node else {
        return Vec::new();
    };
    let mut names = Vec::new();
    collect_self_assign_names(source, root, &mut names);
    names
}

fn collect_self_assign_names(source: &SourceFile, node: Node<'_>, out: &mut Vec<Vec<u8>>) {
    if matches!(node.kind(), "assignment" | "operator_assignment") {
        if let Some(left) = node.child_by_field_name("left") {
            push_self_call_name(source, left, out);
        }
    }
    let mut cur = node.walk();
    for c in node.children(&mut cur) {
        collect_self_assign_names(source, c, out);
    }
}

fn push_self_call_name(source: &SourceFile, mut node: Node<'_>, out: &mut Vec<Vec<u8>>) {
    // Unwrap element_reference / call chain to the self.foo receiver.
    for _ in 0..4 {
        if node.kind() == "element_reference" {
            if let Some(obj) = node
                .child_by_field_name("object")
                .or_else(|| node.child_by_field_name("receiver"))
            {
                node = obj;
                continue;
            }
        }
        break;
    }
    if node.kind() != "call" {
        return;
    }
    if call_receiver(node).is_none_or(|r| r.kind() != "self") {
        return;
    }
    if let Some(method) = call_method_name(source, node) {
        let bare = method.strip_suffix(b"=").unwrap_or(method);
        if !out.iter().any(|n| n == bare) {
            out.push(bare.to_vec());
        }
    }
}

fn self_is_redundant(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    if method == b"class" || method.starts_with(b"[") {
        return false;
    }
    // `self.CONST` / CamelCase — constant-style names aren't RedundantSelf targets.
    if method.first().is_some_and(|b| b.is_ascii_uppercase()) {
        return false;
    }
    // `self.foo =` is never redundant (local would be assignment, not setter call).
    if method.ends_with(b"=") && method != b"==" && method != b"!=" && method != b"=~" && method != b"!~"
    {
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
