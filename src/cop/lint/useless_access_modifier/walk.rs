//! Walk class/module bodies tracking visibility (RuboCop-style).

use tree_sitter::Node;

use super::{
    UselessAccessModifier, Vis, apply_modifier, is_modifier, modifier_vis, report_useless,
};
use crate::cop::shared::call_method_name;
use crate::cop::CopConfig;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

fn string_list(config: &CopConfig, key: &str) -> Vec<String> {
    match config.options.get(key) {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(serde_yml::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn bare_call_named(source: &SourceFile, node: Node<'_>, names: &[&[u8]]) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
        && node.child_by_field_name("receiver").is_none()
        && call_method_name(source, node).is_some_and(|m| names.contains(&m))
}

fn is_method_definition(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() == "method" {
        return true;
    }
    if bare_call_named(
        source,
        node,
        &[
            b"attr",
            b"attr_reader",
            b"attr_writer",
            b"attr_accessor",
            b"define_method",
        ],
    ) {
        return true;
    }
    string_list(config, "MethodCreatingMethods").iter().any(|m| {
        m != "included"
            && matches!(node.kind(), "call" | "command" | "command_call")
            && node.child_by_field_name("receiver").is_none()
            && call_method_name(source, node) == Some(m.as_bytes())
    })
}

fn block_of_call(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .find(|c| matches!(c.kind(), "do_block" | "block"))
}

fn call_block_name<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
) -> Option<(Node<'a>, &'a [u8])> {
    if !matches!(node.kind(), "call" | "command" | "command_call") {
        return None;
    }
    let name = call_method_name(source, node)?;
    block_of_call(node).map(|b| (b, name))
}

fn is_included_block(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    config.get_bool("ActiveSupportExtensionsEnabled", false)
        && call_block_name(source, node).is_some_and(|(_, n)| n == b"included")
}

fn is_new_scope(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if matches!(node.kind(), "class" | "module" | "singleton_class") {
        return true;
    }
    let Some((_block, name)) = call_block_name(source, node) else {
        return false;
    };
    matches!(name, b"class_eval" | b"instance_eval" | b"module_eval")
        || string_list(config, "ContextCreatingMethods")
            .iter()
            .any(|m| m != "included" && name == m.as_bytes())
}

fn body_children<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    if let Some(body) = node.child_by_field_name("body") {
        let mut cur = body.walk();
        return body.named_children(&mut cur).collect();
    }
    if matches!(node.kind(), "do_block" | "block") {
        let mut cur = node.walk();
        if let Some(body) = node
            .named_children(&mut cur)
            .find(|c| matches!(c.kind(), "body_statement" | "block_body"))
        {
            let mut bcur = body.walk();
            return body.named_children(&mut bcur).collect();
        }
    }
    let mut cur = node.walk();
    node.named_children(&mut cur).collect()
}

fn named_kids<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut c = node.walk();
    node.named_children(&mut c).collect()
}

fn check_independent_scope(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let kids = if matches!(node.kind(), "call" | "command" | "command_call") {
        block_of_call(node)
            .map(body_children)
            .unwrap_or_default()
    } else {
        body_children(node)
    };
    let (_, unused) = walk_children(cop, source, &kids, Vis::Public, None, config, diagnostics);
    if let Some(prev) = unused {
        report_useless(cop, source, prev, diagnostics);
    }
}

fn descend<'a>(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'a>,
    cur: Vis,
    unused: Option<Node<'a>>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Vis, Option<Node<'a>>) {
    let nested = body_children(child);
    let kids = if nested.is_empty() {
        named_kids(child)
    } else {
        nested
    };
    if kids.is_empty() {
        return (cur, unused);
    }
    walk_children(cop, source, &kids, cur, unused, config, diagnostics)
}

fn walk_one<'a>(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'a>,
    cur: Vis,
    unused: Option<Node<'a>>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Vis, Option<Node<'a>>) {
    if let Some(v) = modifier_vis(source, child).filter(|_| is_modifier(source, child)) {
        return apply_modifier(cop, source, child, v, cur, unused, diagnostics);
    }
    if is_included_block(source, child, config) {
        return (cur, unused);
    }
    if is_method_definition(source, child, config) {
        return (cur, None);
    }
    if is_new_scope(source, child, config) {
        if !matches!(child.kind(), "class" | "module" | "singleton_class") {
            check_independent_scope(cop, source, child, config, diagnostics);
        }
        return (cur, unused);
    }
    if child.kind() == "singleton_method" {
        return (cur, unused);
    }
    descend(cop, source, child, cur, unused, config, diagnostics)
}

fn walk_children<'a>(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    children: &[Node<'a>],
    mut cur: Vis,
    mut unused: Option<Node<'a>>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Vis, Option<Node<'a>>) {
    for &child in children {
        (cur, unused) = walk_one(cop, source, child, cur, unused, config, diagnostics);
    }
    (cur, unused)
}

pub(super) fn scan_body(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    body: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (_, unused) = walk_children(
        cop,
        source,
        &named_kids(body),
        Vis::Public,
        None,
        config,
        diagnostics,
    );
    if let Some(prev) = unused {
        report_useless(cop, source, prev, diagnostics);
    }
}
