//! Naming/VariableName — locals/ivars snake_case (EnforcedStyle).

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct VariableName;

// Locals, `@ivar`, `@@cvar` — RuboCop strips sigils when matching style.
static SNAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A(?:@@|@)?[\p{Ll}\d_]+[!?=]?\z").unwrap());

fn is_method_name(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "call" | "command" | "command_call" | "method" | "singleton_method" => parent
            .child_by_field_name("method")
            .or_else(|| parent.child_by_field_name("name"))
            .is_some_and(|m| m.id() == node.id()),
        _ => false,
    }
}

/// Pattern-match bindings (`match_var` in Parser) — RuboCop does not style-check them.
fn is_pattern_binding(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind() {
        "keyword_pattern" => parent
            .child_by_field_name("value")
            .is_some_and(|v| v.id() == node.id()),
        "array_pattern" | "find_pattern" | "parenthesized_pattern" => true,
        "splat_parameter" | "hash_splat_parameter" => true,
        "match_pattern" | "match_pattern_p" => parent
            .child_by_field_name("name")
            .is_some_and(|n| n.id() == node.id()),
        _ => false,
    }
}

fn is_assign_lhs(node: Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if matches!(parent.kind(), "assignment" | "operator_assignment") {
        return parent
            .child_by_field_name("left")
            .is_some_and(|l| l.id() == node.id());
    }
    let mut p = Some(parent);
    while let Some(cur) = p {
        if cur.kind() == "left_assignment_list" {
            return true;
        }
        if matches!(cur.kind(), "assignment" | "operator_assignment") {
            break;
        }
        p = cur.parent();
    }
    false
}

fn should_check_ident(node: Node<'_>) -> bool {
    node.kind() == "identifier" && !is_method_name(node) && !is_pattern_binding(node)
}

fn bare_name(raw: &str) -> &str {
    raw.strip_prefix("@@")
        .or_else(|| raw.strip_prefix('@'))
        .unwrap_or(raw)
}

fn style_ok(name: &str, style: &str) -> bool {
    if name == "_" {
        return true;
    }
    match style {
        "camelCase" => is_lower_camel(name),
        _ => SNAKE.is_match(name),
    }
}

fn is_lower_camel(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return true;
    };
    let start = if first == '_' {
        match chars.next() {
            None => return true,
            Some(c) => c,
        }
    } else {
        first
    };
    if !start.is_lowercase() {
        return false;
    }
    chars.all(|c| c.is_lowercase() || c.is_uppercase() || c.is_ascii_digit())
}

fn string_list(config: &CopConfig, key: &str) -> Vec<String> {
    match config.options.get(key) {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(serde_yml::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn allowed(config: &CopConfig, name: &str) -> bool {
    if string_list(config, "AllowedIdentifiers")
        .iter()
        .any(|a| a == name)
    {
        return true;
    }
    for p in string_list(config, "AllowedPatterns") {
        if let Ok(re) = Regex::new(&p) {
            if re.is_match(name) {
                return true;
            }
        }
    }
    false
}

fn forbidden_msg(name: &str) -> String {
    format!("`{name}` is forbidden, use another variable name instead.")
}

fn check_forbidden(
    cop: &VariableName,
    source: &SourceFile,
    name: &str,
    line: usize,
    column: usize,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if string_list(config, "ForbiddenIdentifiers")
        .iter()
        .any(|f| f == name)
    {
        diagnostics.push(cop.diagnostic(source, line, column, forbidden_msg(name)));
    }
    for p in string_list(config, "ForbiddenPatterns") {
        if let Ok(re) = Regex::new(&p) {
            if re.is_match(name) {
                diagnostics.push(cop.diagnostic(source, line, column, forbidden_msg(name)));
            }
        }
    }
}

fn check_name(
    cop: &VariableName,
    source: &SourceFile,
    name_node: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let raw = String::from_utf8_lossy(node_bytes(source, name_node));
    let name = bare_name(&raw);
    let (line, column) = source.offset_to_line_col(name_node.start_byte());
    check_forbidden(cop, source, name, line, column, config, diagnostics);
    if allowed(config, name) {
        return;
    }
    let style = config.get_str("EnforcedStyle", "snake_case");
    if style_ok(name, style) {
        return;
    }
    let style_msg = if style == "camelCase" {
        "camelCase"
    } else {
        "snake_case"
    };
    diagnostics.push(cop.diagnostic(
        source,
        line,
        column,
        format!("Use {style_msg} for variable names."),
    ));
}

impl Cop for VariableName {
    fn name(&self) -> &'static str {
        "Naming/VariableName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[
            "assignment",
            "operator_assignment",
            "identifier",
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        match node.kind() {
            "assignment" | "operator_assignment" => {
                let Some(left) = node.child_by_field_name("left") else {
                    return;
                };
                if matches!(
                    left.kind(),
                    "identifier" | "instance_variable" | "class_variable"
                ) {
                    check_name(self, source, left, config, diagnostics);
                }
            }
            "identifier" => {
                if !should_check_ident(node) || is_assign_lhs(node) {
                    return;
                }
                check_name(self, source, node, config, diagnostics);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(VariableName, "cops/naming/variable_name");
}
