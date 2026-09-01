//! Local / parameter shadowing checks for RedundantSelf.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

pub(super) fn name_is_shadowed(source: &SourceFile, node: Node<'_>, method: &[u8]) -> bool {
    let bare = method.strip_suffix(b"=").unwrap_or(method);
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "method" | "singleton_method" | "block" | "do_block" | "lambda") {
            if params_include(source, n, bare) || lvars_before(source, n, node, bare) {
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
        if is_paramish(child) && param_tree_matches(source, child, name) {
            return true;
        }
    }
    false
}

fn is_paramish(child: Node<'_>) -> bool {
    matches!(
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
    )
}

fn param_tree_matches(source: &SourceFile, child: Node<'_>, name: &[u8]) -> bool {
    if param_name_match(source, child, name) {
        return true;
    }
    let mut c2 = child.walk();
    child
        .named_children(&mut c2)
        .any(|grand| param_name_match(source, grand, name))
}

fn param_name_match(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    match node.kind() {
        "identifier" => node_bytes(source, node) == name,
        "optional_parameter" | "keyword_parameter" | "splat_parameter"
        | "hash_splat_parameter" | "block_parameter" => param_ident(source, node, name),
        _ => false,
    }
}

fn param_ident(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    node.child_by_field_name("name")
        .or_else(|| {
            let mut cur = node.walk();
            node.named_children(&mut cur).find(|n| n.kind() == "identifier")
        })
        .is_some_and(|n| node_bytes(source, n) == name)
}

fn lvars_before(source: &SourceFile, scope: Node<'_>, before: Node<'_>, name: &[u8]) -> bool {
    let mut found = false;
    for_each_before(scope, before, &mut |n| {
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

fn for_each_before(root: Node<'_>, before: Node<'_>, f: &mut impl FnMut(Node<'_>)) {
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
