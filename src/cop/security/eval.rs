//! Security/Eval — Kernel.eval / eval calls.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct Eval;

impl Cop for Eval {
    fn name(&self) -> &'static str {
        "Security/Eval"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let method = node
            .child_by_field_name("method")
            .or_else(|| {
                let mut c = node.walk();
                node.children(&mut c).find(|n| n.kind() == "identifier")
            });
        let Some(method) = method else {
            return;
        };
        let name = &source.as_bytes()[method.start_byte()..method.end_byte()];
        if name != b"eval" {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "The use of `eval` is a serious security risk.".to_string(),
        ));
    }
}
