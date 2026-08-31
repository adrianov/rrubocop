//! Style/RaiseArgs — raise args style.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RaiseArgs;

impl Cop for RaiseArgs {
    fn name(&self) -> &'static str {
        "Style/RaiseArgs"
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
        let Some(msg) = raise_style_msg(source, node, config) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(source, line, col, msg));
    }
}

fn raise_style_msg(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> Option<String> {
    if node.child_by_field_name("receiver").is_some() {
        return None;
    }
    let method = raise_method_name(source, node)?;
    let style = config.get_str("EnforcedStyle", "exploded");
    let args = argument_nodes(node);
    if args.is_empty() {
        return None;
    }
    style_message(style, method, &args, source, config)
}

fn raise_method_name(source: &SourceFile, node: Node<'_>) -> Option<&'static str> {
    match call_method_name(source, node)? {
        b"raise" => Some("raise"),
        b"fail" => Some("fail"),
        _ => None,
    }
}

fn style_message(
    style: &str,
    method: &str,
    args: &[Node<'_>],
    source: &SourceFile,
    config: &CopConfig,
) -> Option<String> {
    let compact = is_disallowed_compact(args, source, config);
    let exploded = args.len() >= 2;
    if style == "exploded" && compact {
        Some(format!(
            "Provide an exception class and message as arguments to `{method}`."
        ))
    } else if style == "compact" && exploded {
        Some(format!(
            "Provide an exception object as an argument to `{method}`."
        ))
    } else {
        None
    }
}

fn is_disallowed_compact(args: &[Node<'_>], source: &SourceFile, config: &CopConfig) -> bool {
    if args.len() != 1 || args[0].kind() != "call" {
        return false;
    }
    let new_call = args[0];
    if call_method_name(source, new_call) != Some(b"new") {
        return false;
    }
    if new_call.child_by_field_name("receiver").is_none() {
        return false;
    }
    if allowed_compact_type(source, new_call, config) {
        return false;
    }
    !acceptable_exploded_args(new_call)
}

/// RuboCop allows multi-arg / kwargs / splat / block `.new` (not convertible).
fn acceptable_exploded_args(new_call: Node<'_>) -> bool {
    if has_block(new_call) {
        return true;
    }
    let args = argument_nodes(new_call);
    if args.len() > 1 {
        return true;
    }
    if args.is_empty() {
        return false;
    }
    matches!(
        args[0].kind(),
        "splat_argument"
            | "hash_splat_argument"
            | "forward_argument"
            | "forwarded_rest_argument"
            | "forwarded_keyword_argument"
            | "pair"
            | "hash"
    )
}

fn has_block(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    node.named_children(&mut cur)
        .any(|n| matches!(n.kind(), "block" | "do_block"))
}

fn allowed_compact_type(source: &SourceFile, new_call: Node<'_>, config: &CopConfig) -> bool {
    let Some(recv) = new_call.child_by_field_name("receiver") else {
        return false;
    };
    let type_name = const_path_name(source, recv);
    let Some(allowed) = config.options.get("AllowedCompactTypes") else {
        return false;
    };
    match allowed {
        serde_yml::Value::Sequence(items) => items.iter().any(|v| {
            v.as_str()
                .is_some_and(|s| s.as_bytes() == type_name.as_slice())
        }),
        serde_yml::Value::String(s) => s.as_bytes() == type_name.as_slice(),
        _ => false,
    }
}

fn const_path_name(source: &SourceFile, node: Node<'_>) -> Vec<u8> {
    match node.kind() {
        "constant" => node_bytes(source, node).to_vec(),
        "scope_resolution" => {
            let mut parts = Vec::new();
            if let Some(scope) = node.child_by_field_name("scope") {
                parts.extend(const_path_name(source, scope));
                parts.push(b':');
                parts.push(b':');
            } else {
                parts.extend(b"::");
            }
            if let Some(name) = node.child_by_field_name("name") {
                parts.extend(node_bytes(source, name));
            }
            parts
        }
        _ => node_bytes(source, node).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RaiseArgs, "cops/style/raise_args");
}
