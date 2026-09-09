//! Style/Alias — prefer `alias` or `alias_method` per EnforcedStyle.

mod scope;

use tree_sitter::Node;

use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, method_node, node_bytes, push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use scope::{
    alias_method_value_used, inside_method_def, lexical_scope_label, scope_of, Scope,
};

pub struct Alias;

fn wants_alias_keyword(config: &CopConfig) -> bool {
    config.get_str("EnforcedStyle", "prefer_alias") != "prefer_alias_method"
}

fn alias_args(node: Node<'_>) -> Option<(Node<'_>, Node<'_>)> {
    let mut cur = node.walk();
    let named: Vec<_> = node
        .named_children(&mut cur)
        .filter(|n| n.kind() != "alias")
        .collect();
    (named.len() == 2).then_some((named[0], named[1]))
}

fn is_symbol(node: Node<'_>) -> bool {
    matches!(node.kind(), "simple_symbol" | "symbol" | "delimited_symbol")
}

fn bareword_alias_arg(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "identifier" | "constant" => true,
        "simple_symbol" | "symbol" => !node_bytes(source, node).starts_with(b":"),
        _ => false,
    }
}

fn symbol_ident<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a str> {
    let raw = std::str::from_utf8(node_bytes(source, node)).ok()?;
    Some(raw.strip_prefix(':').unwrap_or(raw))
}

fn correct_to_alias(
    source: &SourceFile,
    node: Node<'_>,
    args: &[Node<'_>],
    corrections: &mut Option<&mut Vec<Correction>>,
) -> bool {
    let (Some(new), Some(old)) = (symbol_ident(source, args[0]), symbol_ident(source, args[1]))
    else {
        return false;
    };
    push_replace(
        corrections,
        node.start_byte(),
        node.end_byte(),
        format!("alias {new} {old}"),
        "Style/Alias",
    )
}

fn two_symbol_args(node: Node<'_>) -> Option<Vec<Node<'_>>> {
    let args = argument_nodes(node);
    (args.len() == 2 && args.iter().all(|a| is_symbol(*a))).then_some(args)
}

fn alias_method_ok(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    call_method_name(source, node) == Some(b"alias_method")
        && call_receiver(node).is_none()
        && wants_alias_keyword(config)
        && scope_of(source, node) != Scope::Dynamic
        && !alias_method_value_used(node)
}

fn alias_method_candidate<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
    config: &CopConfig,
) -> Option<Vec<Node<'a>>> {
    if !alias_method_ok(source, node, config) {
        return None;
    }
    two_symbol_args(node)
}

fn report_prefer_alias(
    cop: &Alias,
    source: &SourceFile,
    node: Node<'_>,
    args: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(method_node(node).unwrap_or(node).start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!(
            "Use `alias` instead of `alias_method` {}.",
            lexical_scope_label(node)
        ),
    );
    if correct_to_alias(source, node, args, corrections) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn check_alias_method(
    cop: &Alias,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(args) = alias_method_candidate(source, node, config) else {
        return;
    };
    report_prefer_alias(cop, source, node, &args, diagnostics, corrections);
}

fn check_alias_keyword(
    cop: &Alias,
    source: &SourceFile,
    node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some((new_id, old_id)) = alias_args(node) else {
        return;
    };
    if new_id.kind() == "global_variable"
        || old_id.kind() == "global_variable"
        || inside_method_def(node)
    {
        return;
    }
    let scope = scope_of(source, node);
    if scope == Scope::InstanceEval {
        return;
    }
    if scope == Scope::Dynamic || !wants_alias_keyword(config) {
        report_prefer_alias_method(cop, source, node, new_id, old_id, diagnostics, corrections);
        return;
    }
    // Bareword args are fine; only flag `alias :a :b`.
    if bareword_alias_arg(source, new_id) || bareword_alias_arg(source, old_id) {
        return;
    }
    report_prefer_bareword(cop, source, new_id, old_id, diagnostics, corrections);
}

fn report_prefer_alias_method(
    cop: &Alias,
    source: &SourceFile,
    node: Node<'_>,
    new_id: Node<'_>,
    old_id: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        "Use `alias_method` instead of `alias`.".into(),
    );
    if let (Some(new), Some(old)) = (symbol_ident(source, new_id), symbol_ident(source, old_id)) {
        if push_replace(
            corrections,
            node.start_byte(),
            node.end_byte(),
            format!("alias_method :{new}, :{old}"),
            cop.name(),
        ) {
            diag.corrected = true;
        }
    }
    diagnostics.push(diag);
}

fn report_prefer_bareword(
    cop: &Alias,
    source: &SourceFile,
    new_id: Node<'_>,
    old_id: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(new) = symbol_ident(source, new_id) else {
        return;
    };
    let Some(old) = symbol_ident(source, old_id) else {
        return;
    };
    let (line, col) = source.offset_to_line_col(new_id.start_byte());
    let current = format!(
        "{} {}",
        std::str::from_utf8(node_bytes(source, new_id)).unwrap_or(new),
        std::str::from_utf8(node_bytes(source, old_id)).unwrap_or(old)
    );
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!("Use `alias {new} {old}` instead of `alias {current}`."),
    );
    if push_replace(
        corrections,
        new_id.start_byte(),
        old_id.end_byte(),
        format!("{new} {old}"),
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for Alias {
    fn name(&self) -> &'static str {
        "Style/Alias"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "alias"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"alias_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        match node.kind() {
            "alias" => {
                check_alias_keyword(self, source, node, config, diagnostics, &mut corrections)
            }
            _ => check_alias_method(self, source, node, config, diagnostics, &mut corrections),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Alias, "cops/style/alias");
}
