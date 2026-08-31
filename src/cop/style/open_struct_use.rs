//! Style/OpenStructUse — avoid OpenStruct.

use tree_sitter::Node;

use crate::cop::shared::{is_const_named, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct OpenStructUse;

impl Cop for OpenStructUse {
    fn name(&self) -> &'static str {
        "Style/OpenStructUse"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["constant", "scope_resolution"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_const_named(source, node, b"OpenStruct") {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Avoid using `OpenStruct`; use `Struct`, `Hash`, a class, or `Data` instead.".to_string(),
        ));
        let _ = node_bytes;
    }
}
