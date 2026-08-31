//! GraphQL/NotAuthorizedNodeType — Node interface types need authorized?.

use tree_sitter::Node;

use super::helpers::{
    class_body_stmts, collect_calls_named, find_method_def, nested_class, superclass_name,
};
use crate::cop::shared::{argument_nodes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct NotAuthorizedNodeType;

impl Cop for NotAuthorizedNodeType {
    fn name(&self) -> &'static str {
        "GraphQL/NotAuthorizedNodeType"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/graphql/types/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if nested_class(node) || safe_base(source, node, config) {
            return;
        }
        if !implements_node(source, node) || has_authorization(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            ".authorized? should be defined for types implementing Node interface.".into(),
        ));
    }
}

fn safe_base(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let Some(sc) = superclass_name(source, node) else {
        return false;
    };
    let safe = super::helpers::config_string_list(config, "SafeBaseClasses");
    let leaf = sc.rsplit("::").next().unwrap_or(&sc);
    safe.iter().any(|s| {
        s == &sc
            || sc.ends_with(&format!("::{s}"))
            || s.ends_with(&format!("::{sc}"))
            || s == leaf
            || s.rsplit("::").next() == Some(leaf)
    })
}

fn implements_node(source: &SourceFile, class_node: Node<'_>) -> bool {
    for call in collect_calls_named(class_node, source, b"implements") {
        if !in_class(call, class_node) {
            continue;
        }
        for arg in argument_nodes(call) {
            let t = node_text(source, arg);
            if t.contains("Relay::Node") || t.ends_with("::Node") && t.contains("GraphQL") {
                return true;
            }
            if t == "Node" {
                return true;
            }
        }
    }
    false
}

fn has_authorization(source: &SourceFile, class_node: Node<'_>) -> bool {
    find_method_def(class_node, source, "authorized?").is_some()
        || singleton_authorized(source, class_node)
        || [b"can_can_action" as &[u8], b"pundit_role"].iter().any(|name| {
            collect_calls_named(class_node, source, name)
                .into_iter()
                .any(|c| in_class(c, class_node))
        })
}

fn singleton_authorized(source: &SourceFile, class_node: Node<'_>) -> bool {
    for stmt in class_body_stmts(class_node) {
        if is_authorized_singleton(source, stmt) {
            return true;
        }
        if stmt.kind() == "singleton_class"
            && class_body_stmts(stmt)
                .into_iter()
                .any(|inner| named_authorized(source, inner))
        {
            return true;
        }
    }
    false
}

fn is_authorized_singleton(source: &SourceFile, stmt: Node<'_>) -> bool {
    stmt.kind() == "singleton_method" && named_authorized(source, stmt)
}

fn named_authorized(source: &SourceFile, node: Node<'_>) -> bool {
    node.child_by_field_name("name")
        .map(|n| node_text(source, n) == "authorized?")
        .unwrap_or(false)
}

fn in_class(call: Node<'_>, class_node: Node<'_>) -> bool {
    let mut p = call.parent();
    while let Some(n) = p {
        if n.kind() == "class" {
            return n.id() == class_node.id();
        }
        p = n.parent();
    }
    false
}
