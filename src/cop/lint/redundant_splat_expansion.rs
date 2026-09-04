use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RedundantSplatExpansion — `*[1,2]` / `*%w[...]` / `*Array.new(...)`.
pub struct RedundantSplatExpansion;

const LIT_KINDS: &[&str] = &[
    "integer",
    "float",
    "string",
    "simple_symbol",
    "true",
    "false",
    "nil",
    "bare_string",
    "bare_symbol",
    "hash",
    "array",
];

fn splat_inner(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.walk();
    node.named_children(&mut cur).next()
}

fn all_literals(inner: Node<'_>) -> bool {
    let mut cur = inner.walk();
    inner
        .named_children(&mut cur)
        .all(|e| LIT_KINDS.contains(&e.kind()))
}

fn is_percent_array(inner: Node<'_>) -> bool {
    matches!(inner.kind(), "string_array" | "symbol_array")
}

fn method_argument(node: Node<'_>) -> bool {
    node.parent()
        .is_some_and(|p| matches!(p.kind(), "argument_list" | "arguments"))
        || node
            .parent()
            .is_some_and(|p| matches!(p.kind(), "call" | "command" | "command_call"))
}

fn part_of_an_array(node: Node<'_>) -> bool {
    node.parent().is_some_and(|p| p.kind() == "array")
}

fn is_array_new(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "call"
        && call_method_name(source, node) == Some(b"new")
        && call_receiver(node).is_some_and(|r| is_const_named(source, r, b"Array"))
}

fn array_new_inside_multi_array(array_new: Node<'_>) -> bool {
    let Some(arr) = array_new.parent().and_then(|s| s.parent()) else {
        return false;
    };
    if arr.kind() != "array" {
        return false;
    }
    let mut cur = arr.walk();
    arr.named_children(&mut cur).count() > 1
}

fn is_assign_context(kind: &str) -> bool {
    matches!(
        kind,
        "assignment" | "operator_assignment" | "left_assignment_list"
    )
}

/// RuboCop flags `*Array.new` only when the splat sits under an assignment
/// (Parser wraps `a = *Array.new` as `(lvasgn (array (splat …)))`; tree-sitter
/// may attach the splat directly to `assignment` or under a call on the RHS).
fn array_new_assignment_context(splat: Node<'_>) -> bool {
    let mut p = splat.parent();
    for _ in 0..4 {
        let Some(n) = p else {
            return false;
        };
        if is_assign_context(n.kind()) {
            return true;
        }
        if matches!(
            n.kind(),
            "method" | "singleton_method" | "class" | "module" | "do_block" | "block" | "lambda"
        ) {
            return false;
        }
        p = n.parent();
    }
    false
}

fn in_when_or_rescue(splat: Node<'_>) -> bool {
    let mut p = splat.parent();
    while let Some(n) = p {
        if matches!(n.kind(), "when" | "rescue" | "exceptions") {
            return true;
        }
        if matches!(n.kind(), "method" | "singleton_method" | "class" | "module") {
            break;
        }
        p = n.parent();
    }
    false
}

fn report(
    cop: &RedundantSplatExpansion,
    source: &SourceFile,
    node: Node<'_>,
    msg: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(source, line, col, msg.to_string()));
}

fn check_array_new(
    cop: &RedundantSplatExpansion,
    source: &SourceFile,
    node: Node<'_>,
    inner: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if !is_array_new(source, inner) {
        return false;
    }
    if in_when_or_rescue(node)
        || array_new_inside_multi_array(inner)
        || !array_new_assignment_context(node)
    {
        return true;
    }
    report(
        cop,
        source,
        node,
        "Replace splat expansion with comma separated values.",
        diagnostics,
    );
    true
}

fn check_bare_literal(
    cop: &RedundantSplatExpansion,
    source: &SourceFile,
    node: Node<'_>,
    inner: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if !matches!(
        inner.kind(),
        "integer" | "float" | "string" | "simple_symbol" | "bare_string" | "bare_symbol"
    ) {
        return false;
    }
    report(
        cop,
        source,
        node,
        "Replace splat expansion with comma separated values.",
        diagnostics,
    );
    true
}

fn check_literal_array(
    cop: &RedundantSplatExpansion,
    source: &SourceFile,
    node: Node<'_>,
    inner: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !matches!(inner.kind(), "array" | "string_array" | "symbol_array") || !all_literals(inner) {
        return;
    }
    if config.get_bool("AllowPercentLiteralArrayArgument", true)
        && is_percent_array(inner)
        && method_argument(node)
        && !part_of_an_array(node)
    {
        return;
    }
    let msg = if method_argument(node) || part_of_an_array(node) {
        "Pass array contents as separate arguments."
    } else {
        "Replace splat expansion with comma separated values."
    };
    report(cop, source, node, msg, diagnostics);
}

impl Cop for RedundantSplatExpansion {
    fn name(&self) -> &'static str {
        "Lint/RedundantSplatExpansion"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["splat_argument", "splat"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(inner) = splat_inner(node) else {
            return;
        };
        if check_array_new(self, source, node, inner, diagnostics) {
            return;
        }
        if check_bare_literal(self, source, node, inner, diagnostics) {
            return;
        }
        check_literal_array(self, source, node, inner, config, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RedundantSplatExpansion, "cops/lint/redundant_splat_expansion");
}
