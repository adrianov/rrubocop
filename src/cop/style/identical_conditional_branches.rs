//! Style/IdenticalConditionalBranches — identical then/else (incl. ternary).

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::style::heuristics::identical_branch_nodes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct IdenticalConditionalBranches;

impl Cop for IdenticalConditionalBranches {
    fn name(&self) -> &'static str {
        "Style/IdenticalConditionalBranches"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["if", "case", "unless", "conditional"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(branches) = identical_branch_nodes(source, node) else {
            return;
        };
        let src = std::str::from_utf8(node_bytes(source, branches[0])).unwrap_or("?");
        for branch in branches {
            let (line, col) = source.offset_to_line_col(branch.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                format!("Move `{src}` out of the conditional."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(IdenticalConditionalBranches, "cops/style/identical_conditional_branches");
}
