use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/Loop — prefer `Kernel#loop` over `begin/end/while|until`.
pub struct Loop;

impl Cop for Loop {
    fn name(&self) -> &'static str {
        "Lint/Loop"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["while_modifier", "until_modifier"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        if body.kind() != "begin" {
            return;
        }
        let Some((line, col)) = modifier_keyword_pos(source, node) else {
            return;
        };
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use `Kernel#loop` with `break` rather than `begin/end/until`(or `while`)."
                .to_string(),
        ));
    }
}

fn modifier_keyword_pos(source: &SourceFile, node: Node<'_>) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        if !child.is_named() {
            let t = &bytes[child.start_byte()..child.end_byte()];
            if t == b"while" || t == b"until" {
                return Some(source.offset_to_line_col(child.start_byte()));
            }
        }
    }
    None
}
