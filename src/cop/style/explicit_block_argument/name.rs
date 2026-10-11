//! Block-parameter name: reuse an existing one, or pick `block` / `block2` / …
//! so the new parameter does not capture a local already used in the method.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::parse::source::SourceFile;

/// Name of an existing `&param`, or `Some(b"")` for anonymous `&`.
pub(super) fn block_param<'a>(source: &'a SourceFile, params: Node<'_>) -> Option<&'a [u8]> {
    let mut cur = params.walk();
    let found = params
        .named_children(&mut cur)
        .find(|c| c.kind() == "block_parameter")?;
    Some(
        found
            .child_by_field_name("name")
            .map(|n| node_bytes(source, n))
            .unwrap_or(b""),
    )
}

/// Existing block parameter, or the first free `block` / `block2` / …
pub(super) fn pass_name(source: &SourceFile, method: Node<'_>) -> Option<String> {
    if let Some(existing) = method
        .child_by_field_name("parameters")
        .and_then(|p| block_param(source, p))
    {
        return Some(String::from_utf8_lossy(existing).into_owned());
    }
    (0..8u32).find_map(|i| {
        let name = if i == 0 {
            "block".to_string()
        } else {
            format!("block{}", i + 1)
        };
        (!name_taken(source, method, name.as_bytes())).then_some(name)
    })
}

fn name_taken(source: &SourceFile, method: Node<'_>, name: &[u8]) -> bool {
    let mut cur = method.walk();
    method.named_children(&mut cur).any(|child| {
        method
            .child_by_field_name("name")
            .is_none_or(|n| n.id() != child.id())
            && region_uses(source, child, name)
    })
}

fn region_uses(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if matches!(
        node.kind(),
        "method" | "singleton_method" | "class" | "module" | "singleton_class"
    ) || block_binds(source, node, name)
    {
        return false;
    }
    if node.kind() == "identifier" && node_bytes(source, node) == name && !method_ident(node) {
        return true;
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|child| region_uses(source, child, name))
}

fn method_ident(node: Node<'_>) -> bool {
    node.parent()
        .and_then(|parent| parent.child_by_field_name("method"))
        .is_some_and(|method| method.id() == node.id())
}

fn block_binds(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if !matches!(node.kind(), "block" | "do_block" | "lambda") {
        return false;
    }
    node.child_by_field_name("parameters")
        .is_some_and(|params| binds_name(source, params, name))
}

fn binds_name(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    if node.kind() == "identifier" {
        return node_bytes(source, node) == name;
    }
    if matches!(
        node.kind(),
        "optional_parameter"
            | "keyword_parameter"
            | "splat_parameter"
            | "hash_splat_parameter"
            | "block_parameter"
    ) {
        return node
            .child_by_field_name("name")
            .is_some_and(|n| node_bytes(source, n) == name);
    }
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|child| binds_name(source, child, name))
}
