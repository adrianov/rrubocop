//! Style/MethodDefParentheses.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MethodDefParentheses;

impl Cop for MethodDefParentheses {
    fn name(&self) -> &'static str {
        "Style/MethodDefParentheses"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "require_parentheses");
        let has_parens = has_open_paren(source, node);
        let has_params = node.child_by_field_name("parameters").is_some();
        let msg = match style {
            "require_parentheses" if has_params && !has_parens => {
                "Use parentheses for method definitions with parameters."
            }
            "require_no_parentheses" if has_parens => {
                "Omit parentheses for method definitions."
            }
            _ => return,
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg.to_string()));
    }
}

fn has_open_paren(source: &SourceFile, node: Node<'_>) -> bool {
    let params = node.child_by_field_name("parameters");
    let target = params.unwrap_or(node);
    let mut cur = target.walk();
    target
        .children(&mut cur)
        .any(|c| !c.is_named() && node_bytes(source, c) == b"(")
}
