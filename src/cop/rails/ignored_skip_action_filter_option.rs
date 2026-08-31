//! Rails/IgnoredSkipActionFilterOption — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct IgnoredSkipActionFilterOption;

const MSG: &str = "`if` option will be ignored when `only` and `if` are used together.";

impl Cop for IgnoredSkipActionFilterOption {
    fn name(&self) -> &'static str {
        "Rails/IgnoredSkipActionFilterOption"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/app/controllers/**/*.rb", "**/app/mailers/**/*.rb"]
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
        const METHODS: &[&[u8]] = &[b"skip_after_action", b"skip_around_action", b"skip_before_action", b"skip_action_callback", b"if", b"only"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
