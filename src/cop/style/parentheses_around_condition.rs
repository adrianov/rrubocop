//! Style/ParenthesesAroundCondition.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ParenthesesAroundCondition;

impl Cop for ParenthesesAroundCondition {
    fn name(&self) -> &'static str {
        "Style/ParenthesesAroundCondition"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "if", "unless", "while", "until",
            "if_modifier", "unless_modifier", "while_modifier", "until_modifier",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(cond) = node.child_by_field_name("condition") else {
            return;
        };
        if cond.kind() != "parenthesized_statements" {
            return;
        }
        report(self, source, cond, diagnostics, &mut corrections);
    }
}

fn report(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    cond: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(cond.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        "Don't use parentheses around the condition of an `if`/`unless`/`while`/`until`."
            .to_string(),
    );
    if let Some(corr) = corrections.as_mut() {
        strip_open(cop, source, cond, corr);
        strip_close(cop, source, cond, corr);
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn strip_open(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    cond: Node<'_>,
    corr: &mut Vec<Correction>,
) {
    if source.as_bytes().get(cond.start_byte()) != Some(&b'(') {
        return;
    }
    corr.push(Correction {
        start: cond.start_byte(),
        end: cond.start_byte() + 1,
        replacement: String::new(),
        cop_name: cop.name(),
        cop_index: 0,
    });
}

fn strip_close(
    cop: &ParenthesesAroundCondition,
    source: &SourceFile,
    cond: Node<'_>,
    corr: &mut Vec<Correction>,
) {
    if cond.end_byte() <= cond.start_byte() {
        return;
    }
    if source.as_bytes().get(cond.end_byte() - 1) != Some(&b')') {
        return;
    }
    corr.push(Correction {
        start: cond.end_byte() - 1,
        end: cond.end_byte(),
        replacement: String::new(),
        cop_name: cop.name(),
        cop_index: 0,
    });
}
