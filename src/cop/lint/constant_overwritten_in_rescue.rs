use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ConstantOverwrittenInRescue — `rescue => CONST`.
pub struct ConstantOverwrittenInRescue;

impl Cop for ConstantOverwrittenInRescue {
    fn name(&self) -> &'static str {
        "Lint/ConstantOverwrittenInRescue"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["exception_variable"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut cur = node.walk();
        let Some(var) = node.named_children(&mut cur).next() else {
            return;
        };
        if var.kind() != "constant" && var.kind() != "scope_resolution" {
            return;
        }
        let ref_src = node_text(source, var);
        let (line, col) = source.offset_to_line_col(var.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("`{ref_src}` is overwritten by `rescue =>`."),
        ));
    }
}
