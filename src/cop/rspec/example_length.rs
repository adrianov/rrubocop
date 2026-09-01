//! RSpec/ExampleLength — examples must stay within Max lines.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{
    bare_rspec_call, block_body, call_block, code_lines, is_example, RSPEC_INCLUDE,
};
use crate::cop::shared::method_node;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExampleLength;

impl Cop for ExampleLength {
    fn name(&self) -> &'static str {
        "RSpec/ExampleLength"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = bare_rspec_call(source, node) else {
            return;
        };
        if !is_example(method) {
            return;
        }
        let Some(block) = call_block(node) else {
            return;
        };
        let Some(body) = block_body(block) else {
            return;
        };
        let max = config.get_usize("Max", 5);
        // RuboCop counts lines inside the example body (not the `it`/`end` lines).
        let lines = code_lines(source, body);
        if lines <= max {
            return;
        }
        let meth = method_node(node).unwrap_or(node);
        let (line, col) = source.offset_to_line_col(meth.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Example has too many lines. [{lines}/{max}]"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ExampleLength, "cops/rspec/example_length");
}
