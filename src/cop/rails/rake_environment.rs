//! Rails/RakeEnvironment — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct RakeEnvironment;

const MSG: &str = "Add `:environment` dependency to the rake task.";

impl Cop for RakeEnvironment {
    fn name(&self) -> &'static str {
        "Rails/RakeEnvironment"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/Rakefile", "**/*.rake"]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/lib/capistrano/tasks/**/*.rake"]
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
        const METHODS: &[&[u8]] = &[b"task"];
        if !METHODS.contains(&method) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
    }
}
