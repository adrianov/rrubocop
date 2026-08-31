//! Style/FormatString — prefer format/sprintf/%.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FormatString;

impl Cop for FormatString {
    fn name(&self) -> &'static str {
        "Style/FormatString"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "binary"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "format");
        let Some(msg) = format_style_msg(source, node, style) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}

fn format_style_msg(source: &SourceFile, node: Node<'_>, style: &str) -> Option<String> {
    if node.kind() == "binary" {
        return percent_msg(source, node, style);
    }
    let method = call_method_name(source, node)?;
    let current = match method {
        b"format" => "format",
        b"sprintf" => "sprintf",
        _ => return None,
    };
    if current == style {
        None
    } else {
        Some(format!("Prefer `{style}` over `{current}`."))
    }
}

fn percent_msg(source: &SourceFile, node: Node<'_>, style: &str) -> Option<String> {
    if style == "percent" {
        return None;
    }
    let mut cur = node.walk();
    let kids: Vec<_> = node.children(&mut cur).collect();
    if kids.len() >= 2 && node_bytes(source, kids[1]) == b"%" {
        Some(format!("Prefer `{style}` over `%`."))
    } else {
        None
    }
}
