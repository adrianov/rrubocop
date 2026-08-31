//! Style/RegexpLiteral.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RegexpLiteral;

impl Cop for RegexpLiteral {
    fn name(&self) -> &'static str {
        "Style/RegexpLiteral"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["regex"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "slashes");
        let b = node_bytes(source, node);
        let is_pct = b.starts_with(b"%r");
        let is_slash = b.starts_with(b"/");
        let (line, col) = source.offset_to_line_col(node.start_byte());
        if style == "slashes" && is_pct {
            diagnostics.push(self.diagnostic(
                source, line, col,
                "Use `//` around regular expression.".to_string(),
            ));
        } else if style == "percent_r" && is_slash {
            diagnostics.push(self.diagnostic(
                source, line, col,
                "Use `%r` around regular expression.".to_string(),
            ));
        }
    }
}
