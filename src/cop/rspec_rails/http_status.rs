//! RSpecRails/HttpStatus — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct HttpStatus;

fn style_msg(style: &str, kind: &str) -> Option<&'static str> {
    match style {
        "symbolic" if kind == "integer" => {
            Some("Prefer `:symbol` over numeric value to describe HTTP status code.")
        }
        "numeric" if matches!(kind, "symbol" | "simple_symbol" | "string") => {
            Some("Prefer numeric value over `:symbol`/string to describe HTTP status code.")
        }
        _ => None,
    }
}

impl Cop for HttpStatus {
    fn name(&self) -> &'static str {
        "RSpecRails/HttpStatus"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"have_http_status"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"have_http_status") {
            return;
        }
        let args = argument_nodes(node);
        if args.is_empty() {
            return;
        }
        let Some(msg) = style_msg(config.get_str("EnforcedStyle", "symbolic"), args[0].kind())
        else {
            return;
        };
        let (line, col) = source.offset_to_line_col(args[0].start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg.to_string()));
    }
}
