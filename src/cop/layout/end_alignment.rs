//! Layout/EndAlignment.

use tree_sitter::Node;

use crate::cop::layout::end_align;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndAlignment;

fn base_name(kind: &str) -> &'static str {
    match kind {
        "class" => "class", "module" => "module", "if" => "if", "unless" => "unless",
        "while" => "while", "until" => "until", "case" => "case", "do_block" => "do",
        "begin" => "begin", _ => "def",
    }
}

impl Cop for EndAlignment {
    fn name(&self) -> &'static str { "Layout/EndAlignment" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module", "if", "unless", "while", "until", "case", "do_block", "begin", "method", "singleton_method"]
    }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let _ = config;
        end_align::check_end(
            self, source, node, base_name(node.kind()), diagnostics, &mut corrections,
        );
    }
}
