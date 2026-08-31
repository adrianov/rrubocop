use tree_sitter::Node;

use crate::cop::shared::call_receiver;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/NonLocalExitFromIterator — bare return in chained iterator block.
pub struct NonLocalExitFromIterator;

impl Cop for NonLocalExitFromIterator {
    fn name(&self) -> &'static str {
        "Lint/NonLocalExitFromIterator"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["return"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Named `return` only — anonymous keyword tokens share kind `return`.
        if !node.is_named() || node.named_child_count() > 0 {
            return;
        }
        if !exits_chained_iterator(node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Non-local exit from iterator, without return value. `next`, `break`, `Array#find`, etc. are preferred.".to_string(),
        ));
    }
}

fn exits_chained_iterator(ret: Node<'_>) -> bool {
    let mut p = ret.parent();
    while let Some(n) = p {
        match n.kind() {
            "method" | "singleton_method" | "lambda" => return false,
            "block" | "do_block" => {
                return block_is_chained_iterator(n);
            }
            _ => p = n.parent(),
        }
    }
    false
}

fn block_is_chained_iterator(block: Node<'_>) -> bool {
    let Some(send) = block.parent() else {
        return false;
    };
    if !matches!(send.kind(), "call" | "command") {
        return false;
    }
    // RuboCop: block must take arguments, and the send must be chained.
    if !block_has_args(block) {
        return false;
    }
    call_receiver(send).is_some()
}

fn block_has_args(block: Node<'_>) -> bool {
    block.child_by_field_name("parameters").is_some_and(|p| {
        let mut cur = p.walk();
        p.named_children(&mut cur).next().is_some()
    })
}
