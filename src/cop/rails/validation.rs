//! Rails/Validation — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct Validation;

const MSG: &str = "Use `validates :attr, ...` instead of `...`.";

impl Cop for Validation {
    fn name(&self) -> &'static str {
        "Rails/Validation"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/app/models/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        const METHODS: &[&[u8]] = &[b"validates_acceptance_of", b"validates_confirmation_of", b"validates_exclusion_of", b"validates_format_of", b"validates_inclusion_of", b"validates_length_of", b"validates_numericality_of", b"validates_presence_of", b"validates_size_of", b"validates_uniqueness_of"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
