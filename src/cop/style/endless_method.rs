//! Style/EndlessMethod — endless method style.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndlessMethod;

impl Cop for EndlessMethod {
    fn name(&self) -> &'static str {
        "Style/EndlessMethod"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        // RuboCop only implements `on_def` (instance methods), not `on_defs`.
        &["method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // endless: has `=` after args, no `end`
        let mut cur = node.walk();
        let has_end = node.children(&mut cur).any(|c| node_bytes(source, c) == b"end");
        if has_end {
            return;
        }
        // if no end, likely endless (or error)
        let style = config.get_str("EnforcedStyle", "allow_single_line");
        if style == "allow_always" {
            return;
        }
        if style == "forbid_always"
            || (style == "allow_single_line"
                && node.start_position().row != node.end_position().row)
        {
            let (line, col) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Avoid endless method definitions.".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_offense_singleton_endless() {
        crate::testutil::assert_cop_no_offenses_full(
            &EndlessMethod,
            include_bytes!("../../../tests/fixtures/cops/style/endless_method/no_offense_singleton.rb"),
        );
    }
}
