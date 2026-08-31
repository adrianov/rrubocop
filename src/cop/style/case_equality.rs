//! Style/CaseEquality — avoid ===.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CaseEquality;

fn binary_sides<'a>(node: Node<'a>) -> Option<(Node<'a>, Node<'a>)> {
    let left = node.child_by_field_name("left")?;
    let right = node.child_by_field_name("right")?;
    Some((left, right))
}

fn is_triple_eq(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() == "call" {
        return call_method_name(source, node) == Some(b"===");
    }
    let mut cur = node.walk();
    node.children(&mut cur).any(|c| node_bytes(source, c) == b"===")
}

fn receiver(node: Node<'_>) -> Option<Node<'_>> {
    if node.kind() == "call" {
        return call_receiver(node);
    }
    binary_sides(node).map(|(l, _)| l)
}

/// RuboCop `module_name?`: CamelCase, not ALL_CAPS.
fn is_module_name(source: &SourceFile, n: Node<'_>) -> bool {
    let name_node = if n.kind() == "scope_resolution" {
        n.child_by_field_name("name").unwrap_or(n)
    } else if n.kind() == "constant" {
        n
    } else {
        return false;
    };
    let s = std::str::from_utf8(node_bytes(source, name_node)).unwrap_or("");
    let mut chars = s.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && chars.any(|c| c.is_ascii_lowercase())
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_self_class(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "call"
        && call_receiver(node).is_some_and(|r| r.kind() == "self")
        && call_method_name(source, node) == Some(b"class")
}

fn allowed_receiver(source: &SourceFile, recv: Node<'_>, config: &CopConfig) -> bool {
    if recv.kind() == "regex" {
        return true;
    }
    if recv.kind() == "constant" || recv.kind() == "scope_resolution" {
        if !is_module_name(source, recv) {
            return true;
        }
        return config.get_bool("AllowOnConstant", false);
    }
    config.get_bool("AllowOnSelfClass", false) && is_self_class(source, recv)
}

impl Cop for CaseEquality {
    fn name(&self) -> &'static str {
        "Style/CaseEquality"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["binary", "call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_triple_eq(source, node) {
            return;
        }
        if let Some(recv) = receiver(node) {
            if allowed_receiver(source, recv, config) {
                return;
            }
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid the use of the case equality operator `===`.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use crate::testutil::assert_cop_no_offenses_full_with_config;

    crate::cop_fixture_tests!(CaseEquality, "cops/style/case_equality");

    #[test]
    fn allow_on_constant() {
        let mut config = CopConfig::default();
        config
            .options
            .insert("AllowOnConstant".into(), serde_yml::Value::Bool(true));
        assert_cop_no_offenses_full_with_config(
            &CaseEquality,
            b"x = String === body\n",
            config,
        );
    }
}
