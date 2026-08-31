//! RSpec/BeforeAfterAll — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct BeforeAfterAll;

const MSG: &str = "RSpec/BeforeAfterAll offense.";

impl Cop for BeforeAfterAll {
    fn name(&self) -> &'static str {
        "RSpec/BeforeAfterAll"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*_spec.rb", "**/spec/**/*"]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/spec/spec_helper.rb", "**/spec/rails_helper.rb", "**/spec/support/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "symbol", "command"]
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
        const METHODS: &[&[u8]] = &[b"before", b"after"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
