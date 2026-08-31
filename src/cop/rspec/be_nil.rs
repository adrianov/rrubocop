//! RSpec/BeNil — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BeNil;

fn be_nil_rewrite(style: &str, source: &SourceFile, node: Node<'_>) -> Option<(&'static str, &'static str)> {
    let method = call_method_name(source, node)?;
    if style == "be_nil" {
        if method != b"be" {
            return None;
        }
        let args = argument_nodes(node);
        if args.len() != 1 || args[0].kind() != "nil" {
            return None;
        }
        Some(("Prefer `be_nil` over `be(nil)`.", "be_nil"))
    } else if method == b"be_nil" {
        Some(("Prefer `be(nil)` over `be_nil`.", "be(nil)"))
    } else {
        None
    }
}

impl Cop for BeNil {
    fn name(&self) -> &'static str {
        "RSpec/BeNil"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "be_nil");
        let Some((msg, repl)) = be_nil_rewrite(style, source, node) else {
            return;
        };
        let at = if repl == "be_nil" {
            node
        } else {
            method_node(node).unwrap_or(node)
        };
        let (line, col) = source.offset_to_line_col(at.start_byte());
        let mut diag = self.diagnostic(source, line, col, msg.into());
        if push_replace(
            &mut corrections,
            node.start_byte(),
            node.end_byte(),
            repl,
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
