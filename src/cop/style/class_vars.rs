//! Style/ClassVars — avoid class variables.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ClassVars;

impl Cop for ClassVars {
    fn name(&self) -> &'static str {
        "Style/ClassVars"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["assignment", "operator_assignment", "call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if let Some(msg) = class_var_msg(source, node) {
            let at = class_var_loc(node);
            let (line, col) = source.offset_to_line_col(at);
            diagnostics.push(self.diagnostic(source, line, col, msg));
        }
    }
}

fn class_var_msg(source: &SourceFile, node: Node<'_>) -> Option<String> {
    if node.kind() == "call" || node.kind() == "command" {
        return (call_method_name(source, node) == Some(b"class_variable_set"))
            .then(|| "Replace class var set with class instance var.".to_string());
    }
    let left = node.child_by_field_name("left")?;
    if left.kind() != "class_variable" {
        return None;
    }
    let name = String::from_utf8_lossy(node_bytes(source, left));
    Some(format!("Replace class var `{name}` with a class instance var."))
}

fn class_var_loc(node: Node<'_>) -> usize {
    if node.kind() == "call" || node.kind() == "command" {
        node.start_byte()
    } else {
        node.child_by_field_name("left")
            .map(|n| n.start_byte())
            .unwrap_or_else(|| node.start_byte())
    }
}
