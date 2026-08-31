//! Style/ClassMethods — prefer `def self.x` over `def SomeClass.x`.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassMethods;

impl Cop for ClassMethods {
    fn name(&self) -> &'static str {
        "Style/ClassMethods"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some((byte, msg)) = class_method_offense(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(byte);
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}

fn class_method_offense(source: &SourceFile, node: Node<'_>) -> Option<(usize, String)> {
    let object = node.child_by_field_name("object")?;
    if object.kind() == "self" || inside_singleton_class(node) {
        return None;
    }
    let class_like = enclosing_class_or_module_name(source, node)?;
    if node_bytes(source, object) != class_like.as_slice() {
        return None;
    }
    Some((object.start_byte(), suggest_self_method(source, node, &class_like)))
}

fn suggest_self_method(source: &SourceFile, node: Node<'_>, class_like: &[u8]) -> String {
    let method = node
        .child_by_field_name("name")
        .map(|n| String::from_utf8_lossy(node_bytes(source, n)).into_owned())
        .unwrap_or_else(|| "method".to_string());
    let class_name = String::from_utf8_lossy(class_like);
    format!("Use `self.{method}` instead of `{class_name}.{method}`.")
}

fn enclosing_class_or_module_name(source: &SourceFile, node: Node<'_>) -> Option<Vec<u8>> {
    let mut p = node.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "class" | "module") {
            let name = n.child_by_field_name("name")?;
            return Some(node_bytes(source, name).to_vec());
        }
        p = n.parent();
    }
    None
}

fn inside_singleton_class(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(n) = p {
        if n.kind() == "singleton_class" {
            return true;
        }
        if matches!(n.kind(), "class" | "module") {
            return false;
        }
        p = n.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ClassMethods, "cops/style/class_methods");
}
