//! Performance/UnfreezeString — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct UnfreezeString;

fn empty_string_literal(source: &SourceFile, recv: Node<'_>) -> bool {
    matches!(node_text(source, recv).as_str(), "''" | "\"\"")
}

fn string_new_ok(node: Node<'_>) -> bool {
    use crate::cop::shared::argument_nodes;
    let args = argument_nodes(node);
    match args.as_slice() {
        [] => true,
        [a] => matches!(a.kind(), "string" | "heredoc_beginning"),
        _ => false,
    }
}

fn is_bare_string_const(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "constant" && node_bytes(source, node) == b"String"
}

fn is_unfreeze(source: &SourceFile, node: Node<'_>, ruby_ver: f64) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    let Some(recv) = call_receiver(node) else {
        return false;
    };
    match method {
        b"new" => is_bare_string_const(source, recv) && string_new_ok(node),
        b"to_s" | b"to_str" => is_bare_string_const(source, recv),
        // RuboCop-performance skips `.dup` when TargetRubyVersion > 3.2.
        b"dup" | b"clone" => ruby_ver <= 3.2 && empty_string_literal(source, recv),
        _ => false,
    }
}

impl Cop for UnfreezeString {
    fn name(&self) -> &'static str {
        "Performance/UnfreezeString"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "scope_resolution", "constant", "string", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_unfreeze(source, node, config.get_f64("TargetRubyVersion", 2.7)) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Prefer unary plus to get an unfrozen string literal.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(UnfreezeString, "cops/performance/unfreeze_string");

    #[test]
    fn dup_ok_on_ruby_34() {
        let config = CopConfig {
            options: HashMap::from([(
                "TargetRubyVersion".into(),
                serde_yml::Value::Number(serde_yml::Number::from(3.4)),
            )]),
            ..CopConfig::default()
        };
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &UnfreezeString,
            b"result = ''.dup\n",
            config,
        );
    }
}
