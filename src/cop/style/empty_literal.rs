//! Style/EmptyLiteral — Array.new / Hash.new / String.new without args.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyLiteral;

fn const_name(node: Node<'_>, src: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(&src[node.start_byte()..node.end_byte()]).ok()?;
    Some(text.to_string())
}

impl Cop for EmptyLiteral {
    fn name(&self) -> &'static str {
        "Style/EmptyLiteral"
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let method = node.child_by_field_name("method").or_else(|| {
            // fallback: last identifier child
            let mut c = node.walk();
            node.children(&mut c).find(|n| n.kind() == "identifier")
        });
        let Some(method) = method else {
            return;
        };
        let src = source.as_bytes();
        if const_name(method, src).as_deref() != Some("new") {
            return;
        }
        let receiver = node.child_by_field_name("receiver");
        let Some(receiver) = receiver else {
            return;
        };
        // No arguments
        if node.child_by_field_name("arguments").is_some() {
            return;
        }
        let recv = const_name(receiver, src).unwrap_or_default();
        let replacement = match recv.as_str() {
            "Array" => "[]",
            "Hash" => "{}",
            "String" => "''",
            _ => return,
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Use `{replacement}` instead of `{recv}.new`."),
        );
        if let Some(ref mut corr) = corrections {
            corr.push(crate::correction::Correction {
                start: node.start_byte(),
                end: node.end_byte(),
                replacement: replacement.to_string(),
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
