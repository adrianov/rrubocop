use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/SafeNavigationChain — `x&.y.z` ordinary call after &.
pub struct SafeNavigationChain;

/// Methods that are safe on `nil` (NilClass + common stdlib) — RuboCop NilMethods.
const NIL_METHODS: &[&[u8]] = &[
    b"nil?", b"!", b"==", b"!=", b"=~", b"!~", b"<=>", b"===", b"eql?", b"equal?", b"hash",
    b"to_s", b"to_str", b"to_a", b"to_ary", b"to_h", b"to_hash", b"to_i", b"to_int", b"to_f",
    b"to_r", b"to_c", b"to_d", b"inspect", b"to_enum", b"enum_for", b"is_a?", b"kind_of?",
    b"instance_of?", b"class", b"tap", b"yield_self", b"then", b"display", b"method",
    b"public_method", b"singleton_method", b"define_singleton_method", b"freeze", b"frozen?",
    b"object_id", b"itself", b"__id__", b"__send__", b"send", b"public_send", b"respond_to?",
];

impl Cop for SafeNavigationChain {
    fn name(&self) -> &'static str {
        "Lint/SafeNavigationChain"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(op) = node.child_by_field_name("operator") else {
            return;
        };
        // Flag `.` call whose immediate receiver used `&.` (not deeper chain alone).
        if node_bytes(source, op) != b"." {
            return;
        }
        let Some(recv) = call_receiver(node) else {
            return;
        };
        if !immediate_safe_nav(source, recv) {
            return;
        }
        if call_method_name(source, node).is_some_and(|m| {
            NIL_METHODS.contains(&m) || allowed_method(config, m)
        }) {
            return;
        }
        let (line, col) = source.offset_to_line_col(op.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not chain ordinary method call after safe navigation operator.".to_string(),
        ));
    }
}

fn immediate_safe_nav(source: &SourceFile, recv: Node<'_>) -> bool {
    match recv.kind() {
        "call" => recv
            .child_by_field_name("operator")
            .is_some_and(|op| node_bytes(source, op) == b"&."),
        "parenthesized_statements" => {
            let mut cur = recv.walk();
            recv.named_children(&mut cur)
                .next()
                .is_some_and(|n| immediate_safe_nav(source, n))
        }
        _ => false,
    }
}

fn allowed_method(config: &CopConfig, method: &[u8]) -> bool {
    let Some(allowed) = config.options.get("AllowedMethods") else {
        return matches!(
            method,
            b"present?" | b"blank?" | b"presence" | b"try" | b"try!" | b"in?"
        );
    };
    match allowed {
        serde_yml::Value::Sequence(items) => items
            .iter()
            .any(|v| v.as_str().is_some_and(|s| s.as_bytes() == method)),
        serde_yml::Value::String(s) => s.as_bytes() == method,
        _ => false,
    }
}
