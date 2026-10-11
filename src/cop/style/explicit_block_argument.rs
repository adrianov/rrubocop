//! Style/ExplicitBlockArgument — pass the method's block instead of `yield`.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_explicit_block_argument;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExplicitBlockArgument;

const MSG: &str =
    "Consider using explicit block argument in the surrounding method's signature over `yield`.";

fn host<'a>(node: Node<'a>) -> Node<'a> {
    match node.parent() {
        Some(p) if matches!(p.kind(), "call" | "command" | "command_call") => p,
        _ => node,
    }
}

impl Cop for ExplicitBlockArgument {
    fn name(&self) -> &'static str {
        "Style/ExplicitBlockArgument"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "do_block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !matches_explicit_block_argument(source, node, config) {
            return;
        }
        let at = host(node);
        let (line, col) = source.offset_to_line_col(at.start_byte());
        let mut diag = self.diagnostic(source, line, col, MSG.to_string());
        diag.highlight_length = at.end_byte().saturating_sub(at.start_byte()).max(1);
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(ExplicitBlockArgument, "cops/style/explicit_block_argument");
}
